use std::ptr::write_volatile;

use crate::{FrameBuffer, Pixel, BitMap};

/// Map a bitmap onto another
pub(crate) fn map(from: &BitMap, to: &mut BitMap, x: u32, y: u32) {
	let mut i = (y * to.width + x) as usize;
	let j = 0;
	for _ in 0..from.height {
		for _ in 0..from.width {
			//unsafe { write_volatile(std::ptr::addr_of!(to.pxs[i]) as *mut Pixel, from.pxs[j]) }
			to.pxs[i] = from.pxs[j];
			i += 1;
		}
		i += (to.width - (x + from.width)) as usize;
		i += x as usize;
	}
}

pub(crate) fn map_(from: &BitMap, to: &mut BitMap, x: u32, y: u32) {
	let mut i = (y * to.width + x) as usize;
	for sy in 0..from.height {
		for sx in 0..from.width {
			to.pxs[i] = from.pxs[(sy * from.width + sx) as usize];
			i += 1;
		}
		i += (to.width - (x + from.width)) as usize;
		i += x as usize;
	}
}

/// Fill a bitmap with a singular color
pub(crate) fn fill(b: &mut BitMap, px: Pixel) {
	for x in 0..b.height {
		for y in 0..b.width {
			b.pxs[(x * y) as usize] = px
		}
	}
}

pub(crate) fn draw_rect(
	width: u32,
	height: u32,
	x: u32,
	y: u32,
	px: Pixel,
	fb: &mut FrameBuffer,
	v_info: &fb::var_screeninfo
) {
	let mut i = (y * v_info.xres + x) as usize;
	for _ in 0..height {
		for _ in 0..width {
			fb[i] = px;
			i += 1;
		}
		i += (v_info.xres - (x + width)) as usize;
		i += x as usize;
	}
}

pub(crate) fn draw_rect_stroke(
	width: u32,
	height: u32,
	x: u32,
	y: u32,
	px: Pixel,
	thicknes: u32,
	fb: &mut FrameBuffer,
	v_info: &fb::var_screeninfo
) {
	// top
	draw_rect(width, thicknes, x, y, px, fb, v_info);
	// bot
	draw_rect(width, thicknes, x, y + height - thicknes, px, fb, v_info);
	// left
	draw_rect(thicknes, height, x, y, px, fb, v_info);
	// right
	draw_rect(thicknes, height, x + width - thicknes, y, px, fb, v_info);
}

/// Like [draw_rect_stroke] but draws outwards
pub(crate) fn draw_rect_border(
	width: u32,
	height: u32,
	x: u32,
	y: u32,
	px: Pixel,
	thicknes: u32,
	fb: &mut FrameBuffer,
	v_info: &fb::var_screeninfo
) {
	draw_rect_stroke(width + thicknes * 2, height + thicknes * 2, x - thicknes, y - thicknes, px, thicknes, fb, v_info)
}
