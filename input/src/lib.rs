#![feature(read_buf)]
//! For handling evdev input devices

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::format;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::mem::MaybeUninit;
use std::mem::size_of;
use std::os::fd::AsRawFd;
use std::os::fd::RawFd;
use std::path::Path;
use std::io::Read;
use std::ptr;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;
use std::str::FromStr;
use libc::FD_ISSET;
use libc::FD_SET;
use libc::FD_ZERO;
use libc::input_event;
use libc::ioctl;
use ioctl_macro::_IOR;
use libc::select;
use libc::timeval;

/// https://github.com/torvalds/linux/blob/02de58b24d2e1b2cf947d57205bd2221d897193c/include/linux/input.h#L45-L130
/// As read from `/proc/bus/input/devices`
#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub handlers: Vec<String>,
    pub bitmaps: HashMap<String, Vec<u64>>,
}


/**
   struct input_absinfo - used by EVIOCGABS/EVIOCSABS ioctls
    @value: latest reported value for the axis.
    @minimum: specifies minimum value for the axis.
    @maximum: specifies maximum value for the axis.
    @fuzz: specifies fuzz value that is used to filter noise from
    the event stream.
    @flat: values that are within this value will be discarded by
    joydev interface and reported as 0 instead.
    @resolution: specifies resolution for the values reported 
    The resolution for the size axes (ABS_MT_TOUCH_MAJOR,
    ABS_MT_TOUCH_MINOR, ABS_MT_WIDTH_MAJOR, ABS_MT_WIDTH_MINOR)
    is reported in units per millimeter (units/mm).
    When INPUT_PROP_ACCELEROMETER is set the resolution changes.
    The main axes (ABS_X, ABS_Y, ABS_Z) are then reported in
    units per g (units/g) and in units per degree per second
    (units/deg/s) for rotational axes (ABS_RX, ABS_RY, ABS_RZ).
*/
#[derive(Debug)]
pub struct input_absinfo {
    pub value: u32,
    pub minimum: u32,
    pub maximum: u32,
    pub fuzz: u32,
    pub flat: u32,
    pub resolution: u32,
}

#[derive(Debug)]
pub enum MouseEvent {
    MoveRel(i32, i32),
    ScrollVert(i32),
    ScrollHori(i32),
    Left(bool),
    Right(bool),
    Middle(bool),
    Side(bool),
    Extra(bool),
    Forward(bool),
    Back(bool),
    Task(bool),
}

// TODO: Initialize [Mouse] with a limit,
// ensuring it doen't go off-the-screen.
pub struct Mouse {
    pub file: fs::File,
    pub x: u32,
    pub y: u32,
    pub left: bool,
    pub middle: bool,
    pub right: bool,
}

impl Mouse {

    pub fn has_data(&mut self) -> io::Result<bool> {
        let mut readfds = MaybeUninit::uninit();
        unsafe {
            FD_ZERO(readfds.as_mut_ptr());
            FD_SET(self.file.as_raw_fd(), readfds.as_mut_ptr());
        }
        let mut tv = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let res = unsafe { select(
            self.file.as_raw_fd() + 1,
             readfds.as_mut_ptr(),
             ptr::null_mut(),
             ptr::null_mut(),
             addr_of_mut!(tv),
        ) };
        if res == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { FD_ISSET(self.file.as_raw_fd(), readfds.as_ptr()) })
    }

    /// Read a single [input_event], this will most likely break the state of this device.
    pub fn read_single(&mut self) -> io::Result<input_event> {
        let mut buf: [u8;size_of::<input_event>() ] = [0; size_of::<input_event>()];
        self.file.read_exact(&mut buf)?;
        Ok(unsafe { ptr::read(buf.as_ptr() as *const input_event) })
    }

    fn parse_events(events: &Vec<input_event>) -> Option<MouseEvent> {
        match events[0].type_ {
            EV_KEY => {
                if events.len() != 1 {
                    eprintln!("Unexpected amount of mouse button events")
                }
                if ![0,1].contains(&events[0].value) {
                    eprintln!("A none boolean value for a key?? {:#x} for {:#x}", events[0].value, events[0].code);
                    return None
                }
                let val = if events[0].value == 0 { false } else { true };
                match events[0].code {
                    BTN_LEFT => Some(MouseEvent::Left(val)),
                    BTN_RIGHT => Some(MouseEvent::Right(val)),
                    BTN_MIDDLE => Some(MouseEvent::Middle(val)),
                    BTN_FORWARD => Some(MouseEvent::Forward(val)),
                    BTN_BACK => Some(MouseEvent::Back(val)),
                    BTN_SIDE => Some(MouseEvent::Side(val)),
                    BTN_EXTRA => Some(MouseEvent::Extra(val)),
                    _ => {
                        eprintln!("Unimplented mouse event code: {:#x}", events[0].code);
                        None  
                    } 
                }
            },
            EV_REL => {
                if events.len() > 2 {
                    eprintln!("Unexpected amount of EV_REL events for mouse: {:?}", events.len());
                }
                match events.len() {
                    1 => {
                        match events[0].code {
                            0 => Some(MouseEvent::MoveRel(events[0].value, 0)),
                            1 => Some(MouseEvent::MoveRel(0, events[0].value)),
                            8 => Some(MouseEvent::ScrollVert(events[0].value)),
                            6 => Some(MouseEvent::ScrollHori(events[0].value)),
                            _ => {
                                eprintln!("Unexpected code for relative mouse event: {:#x}", events[0].code);
                                return None
                            }
                        }
                    },
                    2 => {
                        if events[1].type_ != EV_REL {
                            eprintln!("Unexpected combination of mouse events");
                            return None
                        }
                        Some(MouseEvent::MoveRel(events[0].value, events[1].value))
                    },
                    _ => {
                        eprintln!("Unexpected amount of relative mouse events {:?}", events.len());
                        return None
                    }
                }
            },
            _ => {
                eprintln!("Unimplented mouse event type: {:#x}", events[0].type_);
                None
            } 
        }
    }

    // /lib/modules/(uname -r)/build/include/uapi/linux/input-event-codes.h
    // https://www.kernel.org/doc/html/latest/input/event-codes.html#input-event-codes
    /// Read events by this mouse.
    /// This can read multiple events until encounters a SYN marker.
    /// An unimplemented events or unexpected properties of events result in a None option,
    /// and a warning in the log.
    pub fn read(&mut self) -> io::Result<Option<MouseEvent>> {
        let mut events: Vec<input_event> = vec![];

        loop {
            let event = self.read_single()?;
            if event.type_ == EV_SYN {
                break
            } else {
                events.push(event);
            }
        }

        if events.len() == 0 {
            eprintln!("No mouse events read, weird");
            return Ok(None)
        }

        let event: Option<MouseEvent> = Mouse::parse_events(&events);

        if let Some(MouseEvent::MoveRel(x, y)) = event {
            self.x = self.x.saturating_add_signed(x);
            self.y = self.y.saturating_add_signed(y);
        }

        Ok(event)
    }

    /// Set the location of this mouse
    pub fn set(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }

    // TODO this doesn't work right because the mouse should really at least have a reference to its name
    /// Create a [Mouse] knowing the evdev file.
    pub fn from_evdev<S: AsRef<OsStr> + ?Sized>(evdev: &S) -> io::Result<Self> {
        let file = fs::File::open(Path::new("/dev/input").join(Path::new(&evdev)))?;
        Ok(Mouse { file, x: 0, y: 0, left: false, middle: false, right: false })
    }
}

pub fn get_abs(fd: RawFd, axis: u8) -> input_absinfo {
    let mut absinfo: MaybeUninit<input_absinfo> = MaybeUninit::uninit();

    //#define EVIOCGABS(abs) _IOR('E', 0x40 + (abs), struct input_absinfo) /* get abs value/limits */
    unsafe { ioctl(fd, _IOR!('E', 0x40 + axis, input_absinfo), absinfo.as_mut_ptr()) };

    unsafe { absinfo.assume_init() }
}

/// Search for a mouse device and return it.
/// Returns [ErrorKind::Other] if no mouse is found.
pub fn mouse() -> io::Result<Mouse> {
    let list = list()?;
    // Detect the mouse by finding a mouse with REL axes, no ABS axes and a mouse button.
    // As inspired by udev:
    // https://github.com/systemd/systemd/blob/e592bf5d11b9d41f77cc72a05c447ea34b787a9e/src/udev/udev-builtin-input_id.c#L266-L270
    let dev = list.iter().find(|d|
        // Does have relative axis
        d.bitmaps.contains_key("REL")
        // Does not have absolute axis
        && !d.bitmaps.contains_key("ABS")
        // Contains a mouse button
        // Using these kernel compatible macros:
        // https://github.com/systemd/systemd/blob/e592bf5d11b9d41f77cc72a05c447ea34b787a9e/src/udev/udev-builtin-input_id.c#L23-L29
        && d.bitmaps.get("KEY").unwrap_or(&vec![0]).iter().any(|k| (k >> (BTN_MOUSE % 64)) & 1 > 0) 
    ).ok_or(
        io::Error::new(ErrorKind::NotFound, "Mouse not found")
    )?;
    let handler = dev.handlers.iter().find(|h| h.starts_with("event")).ok_or(
        io::Error::new(ErrorKind::NotFound, "No evdev handler found")
    )?;
    Mouse::from_evdev(handler)
}

/// Lists devices found in /proc/bus/input/devices
/// https://github.com/torvalds/linux/blob/02de58b24d2e1b2cf947d57205bd2221d897193c/include/linux/input.h#L45
pub fn list() -> io::Result<Vec<Device>> {
    let devices_str = fs::read_to_string("/proc/bus/input/devices")?;
    let mut devices = vec![];
    
    for dev in devices_str.split("\n\n") {
        if dev.trim().is_empty() {
            continue;
        }

        let error = |s| io::Error::other(format!("Parsing failure on {}", s));

        let mut name = String::new();
        let mut handlers = vec![];
        let mut bitmaps: HashMap<String, Vec<u64>> = HashMap::new();

        // https://unix.stackexchange.com/questions/74903/explain-ev-in-proc-bus-input-devices-data
        // https://github.com/torvalds/linux/blob/02de58b24d2e1b2cf947d57205bd2221d897193c/include/linux/input.h#L45
        for line in dev.lines() {
            if line.starts_with("N") {
                name = line
                .strip_prefix("N: Name=\"")
                .ok_or(error("Name"))?
                .strip_suffix("\"")
                .ok_or(error("Name"))?
                .trim().to_owned();
            }
            if line.starts_with("H") {
                handlers = line
                .strip_prefix("H: Handlers=")
                .ok_or(error("Handlers"))?
                .to_owned().trim()
                .split_ascii_whitespace().into_iter().map(|s| s.to_owned()).collect();
            }

            if line.starts_with("B") {
                let (name, values) = line
                .strip_prefix("B: ")
                .ok_or(error("Bitmap"))?
                .trim()
                .split_once('=')
                .ok_or(error("Bitmap"))?;
                let values: Vec<_> = values.split_ascii_whitespace()
                .map(|v| u64::from_str_radix(v, 16)).collect();
                if values.iter().any(|v| v.is_err()) {
                    return Err(error("Bitmap"));
                } else {
                    let values = values.iter().map(|v| v.clone().unwrap()).collect();
                    bitmaps.insert(name.to_owned(), values);
                }
                
            }
        }
        devices.push(Device { name, handlers, bitmaps })
    }

    Ok(devices)
}

// The following is from linux/input-event-codes.h

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const EV_ABS: u16 = 0x03;
const EV_MSC: u16 = 0x04;
const EV_SW: u16 = 0x05;
const EV_LED: u16 = 0x11;
const EV_SND: u16 = 0x12;
const EV_REP: u16 = 0x14;
const EV_FF: u16 = 0x15;
const EV_PWR: u16 = 0x16;
const EV_FF_STATUS: u16 = 0x17;
const EV_MAX: u16 = 0x1f;
const EV_CNT: u16 = EV_MAX + 1;

// Mouse buttons
const BTN_MOUSE: u16 = 0x110;
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const BTN_SIDE: u16 = 0x113;
const BTN_EXTRA: u16 = 0x114;
const BTN_FORWARD: u16 = 0x115;
const BTN_BACK: u16 = 0x116;
const BTN_TASK: u16 = 0x117;

