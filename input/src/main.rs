use std::os::fd::{AsFd, AsRawFd};
use clap::{Parser, Subcommand};

// https://www.kernel.org/doc/html/latest/input/index.html
use input::{mouse, list};

#[derive(Parser)]
struct Args {
    /// Subcommand
    #[command(subcommand)]
    mode: Option<Mode>
}

#[derive(Subcommand)]
enum Mode {
    List,
    Mouse,
}

fn main() {
    let args = Args::parse();
    match args.mode {
        Some(Mode::Mouse) => {
            let mut mouse = input::mouse().unwrap();
            println!("Found mouse at {:?}", mouse.file);
            loop {
                println!("{:?} x: {}, y: {}", mouse.read().unwrap(), mouse.x, mouse.y);
            }
        },
        None | Some(Mode::List) => {
            println!("{:#?}", list().unwrap());
        },
    }

}
