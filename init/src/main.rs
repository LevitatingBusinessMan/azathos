#![feature(panic_backtrace_config)]
#![feature(let_chains)]
use std::{ffi::CString, fmt::Display, os::unix::{prelude::MetadataExt, process::CommandExt}, path::{Path, PathBuf}};
use anyhow::*;
use serde::Deserialize;
use color::{green, red};

static CONFIG_FILE: &'static str = "/etc/init.toml";

#[derive(Deserialize)]
struct Config {
    login: Login,
    mounts: Vec<Mount>,
}

#[derive(Deserialize)]
struct Login {
    shell: String,
    user: String,
    home: String,
    cwd: String,
}

#[derive(Deserialize)]
struct Mount {
    src: String,
    dst: String,
    #[serde(alias = "type")]
    type_: String,
    flags: String,
}

impl Mount {
    pub unsafe fn mount(&self) -> std::result::Result<(), std::io::Error> {
        if libc::mount(
            CString::new(self.src.clone())?.as_ptr(),
            CString::new(self.dst.clone())?.as_ptr(),
            CString::new(self.type_.clone())?.as_ptr(),
            0, std::ptr::null())
        < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            std::io::Result::Ok(())
        }
    }
}

macro_rules! log {
    ($msg:expr, $expr:expr) => {{
        print!("{}... ", $msg);
        let result = $expr;
        if result.is_ok() {
            println!("{}", green!("done"))
        } else {
             println!("{}", red!("failed"))
        }
        result
    }};
}

static  DEFAULT_PATH: &'static str = "/bin /guest/bin";

fn main() {
    println!("Init started");

    match init() {
        Result::Ok(_) => loop { std::thread::park() },
        Result::Err(e) => {
            println!("{e}");
            loop { std::thread::park() }
        }
    }
}

fn init() -> Result<()> {
    let config: Config = log!("Reading config file", {
        let config_file = std::fs::read_to_string(CONFIG_FILE)?;
        toml::from_str::<Config>(&config_file)
    })?;

    mounts(config.mounts);

    println!("Forking off a shell. Stay safe!");
    std::process::Command::new(&config.login.shell)
        .env("HOME", &config.login.home)
        .env("USER", &config.login.user)
        .env("SHELL", &config.login.shell)
        .env("PATH", DEFAULT_PATH)
        .current_dir(&config.login.cwd)
        .spawn()?;
    Ok(())
}

fn mounts(mounts: Vec<Mount>) {
    for mount in mounts {
        if let Err(e) = log!(format!("Mounting {}", mount.src), unsafe { mount.mount() }) {
            println!("{}", e);
        }
    }
}
