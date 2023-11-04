use crate::{draw::draw_rect, Pixel, FrameBuffer};

pub(crate) fn draw_cursor(x: u32, y: u32, fb: &mut FrameBuffer, v_info: &fb::var_screeninfo) {
    draw_rect(10, 10, x, y, Pixel::new( 0x59 , 0x95, 0x9e), fb, v_info);
}
