#![feature(fs_try_exists)]
use clap::Parser;
use libc::{open, ioctl, mmap, PROT_WRITE, MAP_SHARED, munmap, c_void, close, syncfs, socket, AF_UNIX, SOCK_STREAM, bind, sockaddr};
use fb;
use std::{io::{self, Write}, ffi::CString, mem::{MaybeUninit, size_of}, ptr::{null, null_mut, self, addr_of, write_volatile}, process::exit, time::Duration, thread, env, fs, path::Path, os::fd::{AsFd, IntoRawFd, AsRawFd}, rc::Rc, cell::RefCell, borrow::Borrow, sync::{self, atomic::{AtomicBool, AtomicU64}}};

mod window;
use window::Window;

mod draw;

mod cursor;

mod ppf;

#[derive(Parser)]
struct Args {

}

macro_rules! c {
    ($l:literal) => {
        concat!($l, "\x00").as_ptr() as *const i8
    };
}

static SOCK_FILE: &'static str = "display.sock";

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
struct Pixel {
    blue: u8,
    green: u8,
    red: u8,
    _alpha: u8,
}

impl Pixel {
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { _alpha: 0x00, red, green, blue }
    }
    /// Blend with another pixel
    pub fn blend(&self, px: Pixel) -> Pixel {
        if px._alpha == 0xff { return *self }
        if px._alpha == 0x00 { return px }
        unreachable!()
        // let mut s = *self;
        // let factor = (0xff - px._alpha) as f64 / 255 as f64;
        // s.red = self.red.saturating_add((px.red as f64 * factor) as u8);
        // s.green = self.green.saturating_add((px.green as f64 * factor) as u8);
        // s.blue = self.blue.saturating_add((px.blue as f64 * factor) as u8);
        // s
    }
}

type FrameBuffer = [Pixel];

struct BitMap {
    width: u32,
    height: u32,
    pxs: Box<[Pixel]>,
}

impl BitMap {
    /// Get a [BitMap] from the framebuffer
    pub fn from_buffer(fb: &mut [Pixel], width: u32, height: u32) -> Self {
        let pxs: Box<[Pixel]> = unsafe { Box::from_raw(
            ptr::slice_from_raw_parts_mut(
            fb.as_mut_ptr(),
            (height * width * 4).try_into().unwrap()
            )
        ) };
        BitMap {height, width, pxs}
    }

    /// If this bitmap is in-fact a pointer to the framebuffer
    /// it cannot be dropped. So leak it.
    pub fn leak(self) {
        Box::leak(self.pxs);
    }
}

fn configure_vinfo(v_info: &mut fb::var_screeninfo) {
    v_info.bits_per_pixel = 32;
    v_info.xres_virtual = v_info.xres;
    v_info.yres_virtual = v_info.yres;
    v_info.xoffset = 0;
    v_info.yoffset = 0;
}

// pub struct Node {
//     id: usize,
//     parent: Option<usize>,
//     children: Vec<usize>,
// }

// impl Node {
//     fn get<'a, T>(&'a self, arena: &'a Vec<Rc<RefCell<T>>>) -> Option<&Rc<RefCell<T>>> {
//         arena.get(self.id)
//     }
// }

static RENDERING_CURSOR: AtomicBool = AtomicBool::new(false);

static LAST_ID: AtomicU64 = AtomicU64::new(0);

// https://docs.kernel.org/fb/index.html
fn main() {
    let args = Args::parse();
        let fbfile = fs::OpenOptions::new().read(true).write(true).open("/dev/fb0").unwrap_or_else(|e| {
            println!("Error opening framebuffer: {}", e);
            exit(1);
        });

        let v_info: MaybeUninit<fb::var_screeninfo> = MaybeUninit::uninit();
        if unsafe { ioctl(fbfile.as_raw_fd(), fb::IOGET_VSCREENINFO, &v_info) } == -1 {
            println!("Failed to get variable screen info: {}", io::Error::last_os_error());
            exit(1);
        }
        let mut v_info = unsafe { v_info.assume_init() };

        //println!("{:#?}", v_info);
        println!("{}x{}", v_info.xres, v_info.yres);

        configure_vinfo(&mut v_info);

        if unsafe { ioctl(fbfile.as_raw_fd(), fb::IOPUT_VSCREENINFO, &v_info) } == -1 {
            println!("Failed to set variable screen info: {}", io::Error::last_os_error());
            exit(1);
        }

        let size = (v_info.xres_virtual * v_info.yres_virtual * (v_info.bits_per_pixel / 8)) as usize;

        let framebuffer_addr = unsafe { mmap(
            ptr::null_mut(),
            size,
            PROT_WRITE,
            MAP_SHARED,
            fbfile.as_raw_fd(),
            0,     
        ) } as *mut Pixel;
        if framebuffer_addr as i32 == -1 {
            println!("Failed to map framebuffer: {}", io::Error::last_os_error());
        }
        let fb: &mut FrameBuffer = unsafe { std::slice::from_raw_parts_mut(framebuffer_addr, size) };
        
        env::set_current_dir("/tmp").expect("Failed to move to /tmp");
        //let sfd = unsafe { create_socket() };

        // let mut arena: Vec<Rc<RefCell<Window>>> = vec![];
        
        // let root = Node {
        //     id: 0,
        //     parent: None,
        //     children: vec![1],
        // };

        // let win = Node {
        //     id: 1,
        //     parent: Some(0),
        //     children: vec![1],
        // };

        // arena.push(Rc::new(RefCell::new(window::create_root(&v_info))));
        // arena.push(Rc::new(RefCell::new(Window::create(100, 100, 100, 100))));

        fs::OpenOptions::new().write(true).open("/sys/class/graphics/fbcon/cursor_blink").unwrap().write_all(b"0").unwrap();
        fs::OpenOptions::new().write(true).open("/sys/class/vtconsole/vtcon0/bind").unwrap().write_all(b"0").unwrap();

        let mut root = window::create_root(&v_info);
        let mut win = Window::create(100, 100, 100, 100);

        let mut fb_bitmap = BitMap::from_buffer(fb, v_info.xres, v_info.yres);

        root.map(&mut fb_bitmap);
        win.map(&mut fb_bitmap);

        let (tx, rx) = sync::mpsc::channel();

        let mut cursor = cursor::cursor();

        cursor.map_alpha(&mut fb_bitmap);

        thread::Builder::new().name("Render thread".to_string()).spawn(move || {
            loop {
                // the actual rendered thread should drop multiple requests
                // to map the same bitmap (only render the last)
                let (x, y) = rx.recv().unwrap();
                RENDERING_CURSOR.store(true, sync::atomic::Ordering::Relaxed);
                cursor.unmap(&mut fb_bitmap);
                win.unmap(&mut fb_bitmap);
                win.x = x;
                win.y = y;
                cursor.x = x;
                cursor.y = y;
                win.map(&mut fb_bitmap);
                cursor.map_alpha(&mut fb_bitmap);
                RENDERING_CURSOR.store(false, sync::atomic::Ordering::Relaxed);
            }
        }).unwrap();

        let mut mouse = input::mouse().unwrap();
        mouse.set(v_info.xres/2, v_info.yres/2);
        loop {
            if mouse.has_data().unwrap() {
                let mouse_event = mouse.read().unwrap();
                if RENDERING_CURSOR.load(sync::atomic::Ordering::Relaxed) == false {
                    tx.send((mouse.x, mouse.y)).unwrap()
                }
            }
        }

        // let mut mouse = input::mouse().unwrap();
        // mouse.set(v_info.xres/2, v_info.yres/2);
        // loop {
        //     if mouse.has_data().unwrap() {
        //         let mouse_event = mouse.read().unwrap();
        //     }
        // }

        //loop {}

        fb_bitmap.leak();

        if unsafe { munmap(framebuffer_addr as *mut c_void, size) } == -1 {
            println!("Failed to close framebuffer map: {}", io::Error::last_os_error());
        };
}

fn read_socket(sfd: i32) {

}

/// Will create the socket file in current dir
unsafe fn create_socket() -> i32 {
    if SOCK_FILE.len() > 13 {
        println!("Specified socket file path is too long");
        exit(1);
    }

    let sfd = socket(AF_UNIX, SOCK_STREAM, 0);
    if sfd == -1 {
        println!("Failed to create socket: {}", io::Error::last_os_error());
        exit(1);
    }
    let mut sa_data = [0;14];
    let mut i = 0;
    for c in SOCK_FILE.chars() {
        sa_data[i] = c as i8;
        i += 1;
    }
    let addr = sockaddr {
        sa_family: AF_UNIX as u16,
        sa_data: sa_data,
    };
    let socket_path = Path::new("/tmp").join(Path::new(SOCK_FILE));
    if socket_path.exists() {
        fs::remove_file(&socket_path).expect("Failed to remove socket file");
    }
    if bind(sfd, addr_of!(addr), size_of::<sockaddr>() as u32) == -1  {
        println!("Failed to bind socket: {}", io::Error::last_os_error())
    }
    return sfd;
}
