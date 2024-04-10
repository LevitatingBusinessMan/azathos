//! This mostly has to be moved to a genereic gfx library

use crate::{FrameBuffer, Pixel, BitMap};

/// Clone a rectangle from a bitmap
pub(crate) fn get_rect(from: &mut BitMap, x: u32, y: u32, width: u32, height: u32) -> BitMap {
	let mut pxs: Vec<Pixel> = Vec::with_capacity((from.height * from.width) as usize);
	for sy in 0..height {
		for sx in 0..width {
			pxs.push(from.pxs[((sy + y) * from.width + (sx + x)) as usize]);
		}
	}
	BitMap {
		width: width,
		height: height,
		pxs: pxs.into_boxed_slice()
	}
}

/// Map a bitmap onto another
pub(crate) fn map(from: &BitMap, to: &mut BitMap, x: u32, y: u32) {
	let mut i = (y * to.width + x) as usize;
	for sy in 0..from.height {
		for sx in 0..from.width {
			to.pxs[i] = from.pxs[(sy * from.width + sx) as usize];
			i += 1;
		}
		i += (to.width - from.width) as usize;
	}
}

/// Map a bitmap onto another with transparency
pub(crate) fn map_alpha(from: &BitMap, to: &mut BitMap, x: u32, y: u32) {
	let mut i = (y * to.width + x) as usize;
	for sy in 0..from.height {
		for sx in 0..from.width {
			let px = from.pxs[(sy * from.width + sx) as usize];
			if px._alpha != 0xff {
				to.pxs[i] = px;
			}
			//to.pxs[i] = to.pxs[i].blend(from.pxs[(sy * from.width + sx) as usize]);
			i += 1;
		}
		i += (to.width - from.width) as usize;
	}
}

/// Fill a bitmap with a singular color
pub(crate) fn fill(b: &mut BitMap, px: Pixel) {
	for x in 0..b.height {
		for y in 0..b.width {
			b.pxs[(x + y * b.width) as usize] = px
		}
	}
}

/// Draw a solid color rect
pub(crate) fn draw_rect(
	width: u32,
	height: u32,
	x: u32,
	y: u32,
	px: Pixel,
	fb: &mut BitMap,
) {
	let mut i = (y * fb.width + x) as usize;
	for _ in 0..height {
		for _ in 0..width {
			fb.pxs[i] = px;
			i += 1;
		}
		i += (fb.width - (x + width)) as usize;
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
	fb: &mut BitMap,
) {
	// top
	draw_rect(width, thicknes, x, y, px, fb);
	// bot
	draw_rect(width, thicknes, x, y + height - thicknes, px, fb);
	// left
	draw_rect(thicknes, height, x, y, px, fb);
	// right
	draw_rect(thicknes, height, x + width - thicknes, y, px, fb);
}

/// Like [draw_rect_stroke] but draws outwards
pub(crate) fn draw_rect_border(
	width: u32,
	height: u32,
	x: u32,
	y: u32,
	px: Pixel,
	thicknes: u32,
	fb: &mut BitMap,
) {
	draw_rect_stroke(width + thicknes * 2, height + thicknes * 2, x - thicknes, y - thicknes, px, thicknes, fb)
}
