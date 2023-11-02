#![feature(const_trait_impl)]
use clap::Parser;
use libc::{open, ioctl, mmap, PROT_WRITE, MAP_SHARED, munmap, c_void, close, syncfs};
use fb;
use std::{io, ffi::CString, mem::{MaybeUninit, size_of}, ptr::{null, null_mut, self, addr_of, write_volatile}, process::exit, time::Duration};

#[derive(Parser)]
struct Args {

}

macro_rules! c {
    ($l:literal) => {
        concat!($l, "\x00").as_ptr() as *const i8
    };
}


#[repr(C)]
struct Pixel {
    alpha: u8,
    blue: u8,
    green: u8,
    red: u8,
}

type FrameBuffer = [u8];

const MAP_FAILED: i32 = -1;

// https://docs.kernel.org/fb/index.html
fn main() {
    let args = Args::parse();
    unsafe {
        let fbfd = open(c!("/dev/fb0") as *const i8, libc::O_RDWR);
        if fbfd == -1 {
            println!("Error opening framebuffer: {}", io::Error::last_os_error());
            exit(1);
        }
        let v_info: MaybeUninit<fb::var_screeninfo> = MaybeUninit::uninit();
        if ioctl(fbfd, fb::IO_GET_VSCREENINFO, &v_info) == -1 {
            println!("Failed to get variable screen info: {}", io::Error::last_os_error());
            exit(1);
        }
        let v_info = v_info.assume_init();

        println!("{:#?}", v_info);

        if v_info.bits_per_pixel != 32 {
            println!("bpp is not 32");
            exit(1);
        }

        let size = (v_info.xres * v_info.yres * (v_info.bits_per_pixel / 8)) as usize;

        let framebuffer = mmap(
            ptr::null_mut(),
            size,
            PROT_WRITE,
            MAP_SHARED,
            fbfd,
            0,     
        ) as *mut Pixel;
        if framebuffer as i32 == MAP_FAILED {
            println!("Failed to map framebuffer: {}", io::Error::last_os_error());
        }
        let framebuffer = std::slice::from_raw_parts_mut(framebuffer, size);


        std::thread::sleep(Duration::from_secs(1));
        for i in 100..300 {
            for j in 100..300 {
                write_volatile(
                    addr_of!(framebuffer[(v_info.xres * i + j) as usize]) as *mut Pixel,
                    Pixel::new(0xff, 0xff, 0x00, 0x00)
                );
            }
        }

        munmap(addr_of!(framebuffer) as *mut c_void, size);
        close(fbfd);
    }
}

impl Pixel {
    fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self { alpha, red, green, blue }
    }
}
