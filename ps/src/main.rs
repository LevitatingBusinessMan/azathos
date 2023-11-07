use std::{fs, io};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    /// Show full cmdline
    cmdline: bool,
    #[clap(short, long)]
    /// Show state
    state: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        // Filename is a number, so it is a process
        if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
            let cmdline = fs::read_to_string(entry.path().join("cmdline"))?;
            let status  = fs::read_to_string(entry.path().join("status"))?;
            let status = parse_status(&status)?;
            print!("{:<7}", pid);
            if args.state {
                print!(" {}", status.state);
            }
            match args.cmdline {
                true => print!(" {}", cmdline),
                false => print!(" {}", status.name),
            }
            println!()
        }
    }

    Ok(())
}

struct Status {
    name: String,
    state: String,
}

fn parse_status(data: &str) -> io::Result<Status> {
    let error = || io::Error::other("Failed to parse status");
    let mut name = None;
    let mut state = None;
    for line in data.lines() {
        if line.starts_with("Name:") {
            name.replace(line.strip_prefix("Name:")
            .ok_or(error())?
            .trim().to_string());
        }
        if line.starts_with("State:") {
            state.replace(line.strip_prefix("State:")
            .ok_or(error())?
            .split_ascii_whitespace().into_iter().next()
            .ok_or(error())?
            .to_string());
        }
    }
    Ok(Status {
        name: name
        .ok_or(error())?,
        state: state
        .ok_or(error())?,
    })
}
