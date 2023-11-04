#![feature(str_split_whitespace_remainder)]
#![feature(lazy_cell)]
use clap::Parser;
use signal::Signal;
use std::collections::HashMap;
use std::io::{self, Write, Read};
use std::sync::{LazyLock, Mutex};
use color::{green,red};
use std::os::unix::process::*;

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
    let mut status: Option<(bool, Option<i32>)> = None;
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
        }
    }
}

fn read_rc() {
    if let Ok(schelprc) = std::fs::read_to_string(std::path::Path::new("/etc").join(RC_FILENAME)) {
        for line in schelprc.lines() {
            if let Some((cmd, args, background)) = parse(line.trim()) {
                execute(cmd, args, background);
            }
        }
    }
}

fn prompt(status: &Option<(bool, Option<i32>)>, args: &Args) {
    let uid = unsafe { libc::getuid() };
    let mut prompt = if uid == 0 { args.root_prompt.clone() } else { args.user_prompt.clone() };
    if let Some(status) = status {
        match status.0 {
            true => prompt = green!(prompt),
            false => prompt = red!(prompt),
        }
    }
    let cwd = (std::env::current_dir().unwrap().to_string_lossy().to_owned() + " ").to_string();
    print!("{cwd}{prompt} ");
    io::stdout().flush().unwrap();
}

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
                is_string = !is_string
            },
            ' ' | '\n' => {
                if is_string {
                    current_arg.push(c)
                } else if !current_arg.is_empty() {
                    // handle variables
                    if current_arg.starts_with('$') {
                        if let Ok(val) = std::env::var(&current_arg[1..]) {
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

/// Returns None if nothing was executed
fn execute(cmd: String, args: Vec<String>, background: bool) -> Option<(bool, Option<i32>)> {
    // possibly execute as build_in
    if let Some((suc, code)) = build_in(&cmd, &args) {
        save_status(code);
        return Some((suc, code))
    }

    let mut command = std::process::Command::new(&cmd);
    let command = command.args(&args);

    // TODO keep track of backgrounded task
    if background {
        match command.spawn() {
            Ok(_child) => {
                save_status(None);
                None
            },
            Err(e) => {
                save_status(None);
                println!("{cmd}: {e}");
                None
            },
        }
    } else {
        match command.status() {
            Ok(status) => {
                if let Some(signal) = status.signal() {
                    match Signal::try_from(signal) {
                        Ok(sigvar) => println!("{cmd}: Stopped with signal {:?}", sigvar),
                        Err(_) => println!("{cmd}: Stopped with signal {:#x}", signal)
                    }
                }
                save_status(status.code());
                Some((status.success(), status.code()))
            },
            Err(e) => {
                save_status(None);
                println!("{cmd}: {e}");
                None
            },
        }
    }
}

fn build_in(cmd: &str, args: &[String]) -> Option<(bool, Option<i32>)> {
    match cmd {
        "clear" => {
            // https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences
            print!("\x1b[2J"); //clear
            print!("\x1b[0;0H"); //cursor
            Some((true, Some(0)))
        },
        "=" => {
            if args.len() < 2{
                println!("Invalid use of =");
                return Some((false, Some(1)))
            } else {
                std::env::set_var(&args[0], (&args[1..]).join(" ").to_string());
                Some((true, Some(0)))
            }
        },
        "alias" => {
            if args.len() < 2{
                for alias in ALIASES.lock().unwrap().iter() {
                    println!("{} = {}", alias.0, alias.1);
                }
                Some((true, Some(0)))
            } else {
                ALIASES.lock().unwrap().insert(args[0].to_owned(), (&args[1..]).join(" ").to_string());
                Some((true, Some(0)))
            }
        },
        "cd" => {
            let target = if args.is_empty() {
                std::env::var("HOME").unwrap()
            } else {
                args[0].clone()
            };
            match std::env::set_current_dir(target) {
                Ok(_) => Some((true, Some(0))),
                Err(e) => {
                    println!("cd: {e}");
                    return Some((false, Some(e.kind() as i32)))
                },
            }
        },
        "exit" => {
            std::process::exit(0);
        }
        _ => None,
    }
}
