#![feature(panic_backtrace_config)]
use std::{os::unix::{process::CommandExt, prelude::MetadataExt}, ffi::CString};
use anyhow::*;
use serde::Deserialize;
use color::green;

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
    pub unsafe fn mount(&self) -> Result<i32> {
        Ok(libc::mount(
            CString::new(self.src.clone())?.as_ptr(),
            CString::new(self.dst.clone())?.as_ptr(),
            CString::new(self.type_.clone())?.as_ptr(),
            0, std::ptr::null())
        )
    }
}

macro_rules! log {
    ($msg:expr, $expr:expr) => {{
        print!("{}... ", $msg);
        let result = $expr;
        println!("{}", green!("done"));
        result
    }};
}

static  DEFAULT_PATH: &'static str = "/bin";

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
        toml::from_str(&config_file)?
    });

    mounts(config.mounts)?;

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

fn mounts(mounts: Vec<Mount>) -> Result<()> {
    for mount in mounts {
        log!(format!("Mounting {}", mount.src), unsafe {
            let r = mount.mount()?;
            if r != 0 {
                bail!("{}", std::io::Error::last_os_error());
            };
        });
    }
    Ok(())
}
