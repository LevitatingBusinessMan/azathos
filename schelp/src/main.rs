#![feature(str_split_whitespace_remainder)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![feature(fs_try_exists)]
use clap::Parser;
use libc::{WIFEXITED, WIFSIGNALED, WEXITSTATUS, WTERMSIG};
use signal::Signal;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{self, Write, Read};
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::{ptr, fs, env};
use std::sync::{LazyLock, Mutex};
use color::{green,red};

static RC_FILENAME: &'static str = "schelprc";

static ALIASES: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

mod signal;

#[derive(Parser)]
struct Args {
    #[arg(short, default_value="$")]
    user_prompt: String,
    #[arg(short, default_value="#")]
    root_prompt: String,
    #[arg()]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let stdin = io::stdin();
    let mut status: Option<i32> = None;
    read_rc();

    let mut file = args.file.clone().map(|f| {
        let mut f = std::fs::File::open(f).expect("Could not open file");
        let mut buf = String::new();
        f.read_to_string(&mut buf).expect("Failed to read file");
        buf
    });

    let mut file_lines = file.as_mut().map(|f| f.lines());

    loop {
        let mut line: String = String::new();
        match &mut file_lines {
            Some(f) => {
                match f.next() {
                    Some(l) => line = l.to_string(),
                    None => break,
                }
            },
            None => {
                prompt(&status, &args);
                stdin.read_line(&mut line).unwrap();
            },
        }
        if let Some((cmd, args, background)) = parse(line.trim()) {
            status = execute(cmd, args, background);
            save_status(status);
        }
    }
}

fn read_rc() {
    if let Ok(schelprc) = std::fs::read_to_string(std::path::Path::new("/etc").join(RC_FILENAME)) {
        for line in schelprc.lines() {
            if let Some((cmd, args, background)) = parse(line.trim()) {
                let status =  execute(cmd, args, background);
                save_status(status);
            }
        }
    }
}

fn prompt(status: &Option<i32>, args: &Args) {
    let uid = unsafe { libc::getuid() };
    let mut prompt = if uid == 0 { args.root_prompt.clone() } else { args.user_prompt.clone() };
    if let Some(code) = status {
        match *code == 0 {
            true => prompt = green!(prompt),
            false => prompt = red!(prompt),
        }
    }
    let cwd = (std::env::current_dir().unwrap().to_string_lossy().to_owned() + " ").to_string();
    print!("{cwd}{prompt} ");
    io::stdout().flush().unwrap();
}

// FIXME does not handle empty strings properly
fn parse(line: &str) -> Option<(String, Vec<String>, bool)> {
    let mut line = line.to_owned();
    if line.starts_with('#') || line.is_empty() {
        return None
    }
    let mut args = vec![];
    let mut current_arg = String::new();
    let mut is_string = false;

    let mut background = false;

    if line.ends_with('&') {
        background = true;
        line.remove(line.len()-1);
    }

    line.push('\n');

    // check for aliases to expand
    for (alias, expansion) in ALIASES.lock().unwrap().iter() {
        if line.starts_with(&(alias.to_owned() + " ")) || line.starts_with(&(alias.to_owned() + "\n")) {
            line = line.replacen(alias, &expansion, 1);
            break
        }
    }

    for c in line.chars() {
        match c {
            '"' => {
                if is_string && current_arg.is_empty() {
                    args.push("".to_owned());
                }
                is_string = !is_string;
            },
            ' ' | '\n' => {
                if is_string {
                    current_arg.push(c)
                } else if !current_arg.is_empty() {
                    // handle variables
                    if current_arg.starts_with('$') {
                        if let Ok(val) = env::var(&current_arg[1..]) {
                            current_arg = val
                        } else {
                            println!("Variable {} not found", current_arg);
                            return None
                        }
                    }
                    args.push(current_arg);
                    current_arg = String::new();
                }
            }
            _ => current_arg.push(c)
        }
    }

    if is_string {
        println!("Syntax Error: Un-terminated string");
        return None;
    }

    if args.is_empty() {
        return None
    }

    let cmd = args.remove(0);

    return Some((cmd, args, background))
}

// either set or unset the status variable
fn save_status(code: Option<i32>) {
    match code {
        Some(code) => std::env::set_var("status", code.to_string()),
        None => std::env::remove_var("status")
    }
}

fn execute(cmd: String, args: Vec<String>, background: bool) -> Option<i32> {
    // possibly execute as build_in
    if let Some(code) = build_in(&cmd, &args) {
        return Some(code)
    }

    if background {
        println!("Currently, background jobs have been disabled.");
        save_status(None);
        return None
    }

    // Libc is used here instead of Rusts Command and Child struct
    // for better control. It also fits the projects philosophy better.

    // Either use the cmd as a path, or find a path
    let path;
    if let Ok(true) = fs::try_exists(Path::new(&cmd)) {
        path = cmd.clone();
    } else {
        let path_to_use = env::var("PATH").unwrap_or("".to_string()); //+ " .";
        let path_search = path_search(&path_to_use, &cmd);
        if path_search.is_none() {
            eprintln!("schelp: Command {cmd} was not found");
            return None;
        } else {
            path = path_search.unwrap().to_string_lossy().to_string();
        }
    }


    let pid = unsafe { libc::fork() };
    if pid == -1 {
        panic!("Failed to fork!");
    }

    match pid {
        0 => fork_child(path, args),
        _ => fork_parent(pid, &cmd)
    }
}

// TODO have our own nix crate which handles execve and stuff
// with rusty return types
fn fork_child(path: String, args: Vec<String>) -> ! {
    // This isn't exec(3) so we'll have to do PATHs ourselves
    let env = [ptr::null()];

    // Create a clone of args with cmd included as argv
    let mut argv = vec![path.clone()];
    argv.append(&mut args.clone());
    let argv: Vec<CString> = argv.iter().map(|a| CString::new(a.as_str()).unwrap()).collect();
    
    let cmd_cstr = CString::new(path.as_str()).unwrap();
    let mut arg_ptrs: Vec<*const i8> = argv.iter().map(|f| f.as_ptr()).collect();
    arg_ptrs.push(ptr::null());

    let r: i32 = unsafe { libc::execve(
        cmd_cstr.as_ptr(),
        arg_ptrs.as_ptr() as *const *const i8,
        env.as_ptr() as *const *const i8
    ) };
    if r == -1 {
        println!("Failed to execute {path}: {}", io::Error::last_os_error());
        std::process::exit(
            io::Error::last_os_error().raw_os_error()
            .expect("Failed to get raw OS error")
        );
    }
    unreachable!();
}

fn fork_parent(pid: i32, cmd: &str) -> Option<i32> {
    loop {
        let mut wstatus = MaybeUninit::uninit();
        // possibly use WUNTRACED as well, and notify the user if a process was stopped or continued
        let r = unsafe { libc::waitpid(pid, wstatus.as_mut_ptr(), libc::WNOHANG) };
        if r == 0 {
            // Continue if nothing happened
            continue;
        }
        if r == -1 {
            panic!("Waitpid failed!");
        }
        let wstatus = unsafe { wstatus.assume_init() };
        if WIFEXITED(wstatus) {
            let status = WEXITSTATUS(wstatus);
            return Some(status);
        } else if WIFSIGNALED(wstatus) {
            let signal = WTERMSIG(wstatus);
            match Signal::try_from(signal) {
                Ok(sigvar) => println!("{cmd}: Terminated with signal {:?}", sigvar),
                Err(_) => println!("{cmd}: Terminated with signal {:#x}", signal)
            }
            return None;
        }
    }
}


// Returns a code on execution.
// Return none on no execution.
fn build_in(cmd: &str, args: &[String]) -> Option<i32> {
    match cmd {
        "clear" => {
            // https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences
            print!("\x1b[2J"); //clear
            print!("\x1b[0;0H"); //cursor
            Some(0)
        },
        "=" => {
            if args.len() < 2{
                println!("Invalid use of =");
                return Some(1)
            } else {
                std::env::set_var(&args[0], (&args[1..]).join(" ").to_string());
                Some(0)
            }
        },
        "alias" => {
            if args.len() < 2{
                for alias in ALIASES.lock().unwrap().iter() {
                    println!("{} = {}", alias.0, alias.1);
                }
                Some(0)
            } else {
                ALIASES.lock().unwrap().insert(args[0].to_owned(), (&args[1..]).join(" ").to_string());
                Some(0)
            }
        },
        "cd" => {
            let target = if args.is_empty() {
                std::env::var("HOME").unwrap()
            } else {
                args[0].clone()
            };
            match std::env::set_current_dir(target) {
                Ok(_) => Some(0),
                Err(e) => {
                    println!("cd: {e}");
                    return Some(e.kind() as i32)
                },
            }
        },
        "exit" => {
            std::process::exit(0);
        }
        _ => None,
    }
}

// Search path
fn path_search(path: &str, cmd: &str) -> Option<PathBuf> {
    for dir in path.split_ascii_whitespace() {
        match fs::read_dir(Path::new(dir)) {
            Ok(entries) => {
                for ent in entries {
                    if let Ok(ent) = ent && ent.file_name() == cmd {
                        return Some(ent.path());
                    }
                }
            },
            Err(e) => eprintln!("Failed to open {dir} as found in  $PATH: {e}")
        }
    }
    return None;
}

// Old code using [Command] and [Child]
// if background {
//     match command.spawn() {
//         Ok(_child) => {
//             save_status(None);
//             None
//         },
//         Err(e) => {
//             save_status(None);
//             println!("{cmd}: {e}");
//             None
//         },
//     }
// } else {
//     match command.status() {
//         Ok(status) => {
//             if let Some(signal) = status.signal() {
//                 match Signal::try_from(signal) {
//                     Ok(sigvar) => println!("{cmd}: Stopped with signal {:?}", sigvar),
//                     Err(_) => println!("{cmd}: Stopped with signal {:#x}", signal)
//                 }
//             }
//             save_status(status.code());
//             Some((status.success(), status.code()))
//         },
//         Err(e) => {
//             save_status(None);
//             println!("{cmd}: {e}");
//             None
//         },
//     }
// }
