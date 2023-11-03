pub const IOGET_VSCREENINFO: i32 = 0x4600;
pub const IOPUT_VSCREENINFO: i32 = 0x4601;

/// Struct representing screen information for a framebuffer device
#[repr(C)]
#[derive(Debug,Clone)]
pub struct var_screeninfo {
    /// Visible resolution in the x-axis
    pub xres: u32,
    /// Visible resolution in the y-axis
    pub yres: u32,
    /// Virtual resolution in the x-axis
    pub xres_virtual: u32,
    /// Virtual resolution in the y-axis
    pub yres_virtual: u32,
    /// Offset from virtual to visible in the x-axis
    pub xoffset: u32,
    /// Offset from virtual to visible in the y-axis
    pub yoffset: u32,
    /// Bits per pixel
    pub bits_per_pixel: u32,
    /// Grayscale information (0 = color, 1 = grayscale, >1 = FOURCC)
    pub grayscale: u32,
    /// Bitfield for red color if true color, else only length is significant
    pub red: bitfield,
    /// Bitfield for green color if true color, else only length is significant
    pub green: bitfield,
    /// Bitfield for blue color if true color, else only length is significant
    pub blue: bitfield,
    /// Bitfield for transparency
    pub transp: bitfield,
    /// Non-standard pixel format indicator
    pub nonstd: u32,
    /// Activation information (see FB_ACTIVATE_*)
    pub activate: u32,
    /// Height of picture in mm
    pub height: u32,
    /// Width of picture in mm
    pub width: u32,
    /// Acceleration flags (OBSOLETE, see fb_info.flags)
    pub accel_flags: u32,
    /// Pixel clock in picoseconds (ps)
    pub pixclock: u32,
    /// Time from sync to picture on the left side
    pub left_margin: u32,
    /// Time from picture to sync on the right side
    pub right_margin: u32,
    /// Time from sync to picture on the top side
    pub upper_margin: u32,
    /// Time from sync to picture on the bottom side
    pub lower_margin: u32,
    /// Length of horizontal sync
    pub hsync_len: u32,
    /// Length of vertical sync
    pub vsync_len: u32,
    /// Synchronization information (see FB_SYNC_*)
    pub sync: u32,
    /// Video mode information (see FB_VMODE_*)
    pub vmode: u32,
    /// Angle of rotation counter clockwise
    pub rotate: u32,
    /// Colorspace information for FOURCC-based modes
    pub colorspace: u32,
    /// Reserved for future compatibility
    pub reserved: [u32; 4],
}

/// Struct representing a bitfield for color information
#[repr(C)]
#[derive(Debug,Clone)]
pub struct bitfield {
    /// Bit offset
    pub offset: u32,
    /// Bit length
    pub length: u32,
    /// Most significant bit on the right side
    pub msb_right: u32,
}
