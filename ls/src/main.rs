use clap::Parser;
use std::os::unix::prelude::{PermissionsExt, FileTypeExt};
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry, ReadDir};
use color::*;

#[derive(Parser)]
struct Args {
    directory: Option<String>,

    /// Show as a list
    #[arg(short, long)]
    list: bool,
    
    /// Display filetype indicator
    #[arg(short='f', long)]
    classify: bool,
    
    /// Display color
    #[arg(short, long)]
    color: bool,
    
    /// Display size
    #[arg(short, long)]
    size: bool,

    /// Display permissions
    #[arg(short, long)]
    permissions: bool,
}

fn main() {
    let args = Args::parse();
    let dir = match &args.directory {
        Some(dir) => PathBuf::from(dir),
        None => std::env::current_dir().unwrap(),
    };
    match fs::read_dir(&dir) {
        Ok(readdir) => {
            let readdir = readdir.collect();
            let max_size_len = get_max_size_file(&readdir).to_string().len();
            for entry in readdir {
                print_entry(&entry.unwrap(), &args, max_size_len);
            }
            if !args.list {
                println!()
            }
        },
        Err(e) => eprintln!("{e}"),
    }
}

/// Find the maximum filesize in dir
fn get_max_size_file(readdir: &Vec<std::io::Result<DirEntry>>) -> u64 {
    readdir.iter().fold(0, |c, f| {
        match f {
            Ok(f) => match f.metadata() {
                Ok(f) => if f.len() > c {
                    f.len()
                } else {
                    c
                },
                Err(_) => c
            },
            Err(_) => c,
        }
    })
}

fn print_entry(entry: &DirEntry, args: &Args, max_size: usize) {
    let name = entry.file_name();
    let name = name.to_string_lossy();
    let metadata = entry.metadata().unwrap();
    let type_ = metadata.file_type();

    let mut buf = name.to_string();

    // Assign indicators and colors
    if type_.is_dir() {
        if args.color {
            buf = green!(buf);
        } 
        if args.classify {
            buf.push('/');
        }
    } else if type_.is_symlink() {
        if args.color {
            buf = blue!(buf);
        } 
        if args.classify {
            buf.push('~');
        }
        if args.list {
            // TODO: give same colors to link path
            buf += &match fs::read_link(entry.path()) {
                Ok(file) => format!(">{}",file.to_string_lossy()),
                Err(_) => "#".to_owned(),
            }
        }
    } else if type_.is_fifo() {
        if args.color {
            buf = yellow!(buf);
        }
        if args.classify {
            buf.push('|');
        }
    } else if type_.is_socket() {
        if args.color {
            buf = yellow!(buf);
        }
        if args.classify {
            buf.push('=');
        }  
    } else if type_.is_block_device() || type_.is_char_device() {
        if args.color {
            buf = yellow!(buf);
        }
    }  else if metadata.permissions().mode() & 0o111 > 0 {
        if args.color {
            buf = red!(buf);
        } 
        if args.classify {
            buf.push('*');
        }
    }

    match args.list {
        false => print!("{} ", buf),
        true => {
            if args.permissions {
                let permissions = metadata.permissions().mode();
                let permissions = format!(
                    "{}{}{}{}{}{}{}{}{}",
                    if permissions & 0o400 > 0 { 'r' } else { '-' },
                    if permissions & 0o200 > 0 { 'w' } else { '-' },
                    if permissions & 0o100 > 0 { 'x' } else { '-' },
                    if permissions & 0o040 > 0 { 'r' } else { '-' },
                    if permissions & 0o020 > 0 { 'w' } else { '-' },
                    if permissions & 0o010 > 0 { 'x' } else { '-' },
                    if permissions & 0o004 > 0 { 'r' } else { '-' },
                    if permissions & 0o002 > 0 { 'w' } else { '-' },
                    if permissions & 0o001 > 0 { 'x' } else { '-' },
                );
                print!("{permissions} ");
            }
            if args.size {
                print!("{:<max_size$} ", metadata.len());
            }
            println!("{}", buf);
        }
    }
}
