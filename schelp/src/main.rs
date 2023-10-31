#![feature(str_split_whitespace_remainder)]
use clap::Parser;
use std::{io::{self, Write}, process::ExitStatus};
use color::{green,red};

#[derive(Parser)]
struct Args {
    #[arg(short, default_value="$")]
    user_prompt: String,
    #[arg(short, default_value="#")]
    root_prompt: String,
}

fn main() {
    let args = Args::parse();
    let stdin: io::Stdin = io::stdin();
    let mut status: Option<ExitStatus> = None;
    loop {
        prompt(&status, &args);
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();
        status = execute(line.trim());
    }
}

fn prompt(status: &Option<ExitStatus>, args: &Args) {
    let uid = unsafe { libc::getuid() };
    let mut prompt = if uid == 0 { args.root_prompt.clone() } else { args.user_prompt.clone() };
    if let Some(status) = status {
        match status.success() {
            true => prompt = green!(prompt),
            false => prompt = red!(prompt),
        }
    }
    print!("{prompt} ");
    io::stdout().flush().unwrap();
}

fn execute(args: &str) -> Option<ExitStatus> {
    let mut args = args.split_whitespace();
    let command = args.next()?;

    let mut cmd = std::process::Command::new(command);
    cmd.args(args.collect::<Vec<_>>());

    match cmd.status() {
        Ok(status) => return Some(status),
        Err(e) => {
            println!("{e}");
            None
        },
    }
}
