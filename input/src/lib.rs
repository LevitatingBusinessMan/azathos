//! For handling evdev input devices

use std::fs;
use std::io;

#[derive(Debug)]
pub struct Device {
    name: String,
    handlers: Vec<String>
}

/// Lists devices found in /proc/bus/input/devices
pub fn list() -> io::Result<Vec<Device>> {
    let devices_str = fs::read_to_string("/proc/bus/input/devices")?;
    let mut devices = vec![];
    
    for dev in devices_str.split("\n\n") {
        if dev.trim().is_empty() {
            continue;
        }
        let mut name = String::new();
        let mut handlers = vec![];
        for line in dev.lines() {
            if line.starts_with("N") {
                name = line["N: Name=\"".len()..line.len()-1].to_owned()
            }
            if line.starts_with("H") {
                handlers = line["N: Handlers=".len()..line.len()].to_owned().trim().split_ascii_whitespace().into_iter().map(|s| s.to_owned()).collect();
            }
        }
        devices.push(Device { name, handlers })
    }

    Ok(devices)
}
