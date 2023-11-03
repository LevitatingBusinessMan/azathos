#![feature(fs_try_exists)]
use clap::Parser;
use libc::{open, ioctl, mmap, PROT_WRITE, MAP_SHARED, munmap, c_void, close, syncfs, socket, AF_UNIX, SOCK_STREAM, bind, sockaddr};
use fb;
use std::{io, ffi::CString, mem::{MaybeUninit, size_of}, ptr::{null, null_mut, self, addr_of, write_volatile}, process::exit, time::Duration, thread, env, fs, path::Path, os::fd::{AsFd, IntoRawFd, AsRawFd}};

mod window;
use window::Window;

mod draw;

mod cursor;

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
#[derive(Copy, Clone)]
struct Pixel {
    blue: u8,
    green: u8,
    red: u8,
    alpha: u8,
}

type FrameBuffer = [Pixel];

const MAP_FAILED: i32 = -1;

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

        println!("{:#?}", v_info);

        v_info.bits_per_pixel = 32;
        v_info.xres_virtual = v_info.xres;
        v_info.yres_virtual = v_info.yres;
        v_info.xoffset = 0;
        v_info.yoffset = 0;

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
        if framebuffer_addr as i32 == MAP_FAILED {
            println!("Failed to map framebuffer: {}", io::Error::last_os_error());
        }
        let fb: &mut FrameBuffer = unsafe { std::slice::from_raw_parts_mut(framebuffer_addr, size) };
        
        env::set_current_dir("/tmp").expect("Failed to move to /tmp");
        let sfd = unsafe { create_socket() };
    
        let win = Window::create(200,100, Some((20,20)));

        clear_display(&v_info, fb);
        win.draw(fb, &v_info);

        let mut mouse = input::mouse().unwrap();
        mouse.set(v_info.xres/2, v_info.yres/2);
        loop {
            if mouse.has_data().unwrap() {
                let mouse_event = mouse.read().unwrap();
                cursor::draw_cursor(mouse.x, mouse.y, fb, &v_info);
            }
        }

        if unsafe { munmap(framebuffer_addr as *mut c_void, size) } == -1 {
            println!("Failed to close framebuffer map: {}", io::Error::last_os_error());
        };
}

fn read_socket(sfd: i32) {

}

fn clear_display(v_info: &fb::var_screeninfo, framebuffer: &mut FrameBuffer) {
    for i in 0..v_info.yres {
        for j in 0..v_info.xres {
            // Do I need write_volatile?
            framebuffer[(v_info.xres * i + j) as usize] = Pixel::new(0x00, 0xff, 0xff, 0xff);
        }
    }
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

impl Pixel {
    fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self { alpha, red, green, blue }
    }
}
