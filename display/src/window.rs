use std::{rc::Rc, cell::RefCell, sync::{Mutex, Arc, RwLock}};
use slab_tree::{Tree, NodeId, NodeMut};

use crate::{FrameBuffer, Pixel, draw, BitMap};

type WindowId = NodeId;

pub struct Window {
	pub id: u64,
	/// X coordinate, of the underlying bitmap
	pub x: u32,
	/// Y coordinate, of the underlying bitmap
	pub y: u32,
	pub decorated: bool,
	/// The bitmap of the window
	pub bitmap: BitMap,
	/// The bitmap behind the window
	pub backing: Option<BitMap>,
}

const FRAME_BORDER: u32 = 2;
const FRAME_TOP: u32 = 8;

impl Window {
	/// Init a (black) window
	pub fn create(width: u32, height: u32, x: u32, y: u32) -> Self {
		Self {
			id: super::LAST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
			x,
			y,
			decorated: true,
			bitmap: BitMap {
				height,
				width,
				pxs: vec![Pixel::new(0x00, 0x00, 0x00); (height * width) as usize].into_boxed_slice()
			},
			backing: None
		}
	}
	pub fn map(&mut self, fb: &mut BitMap) {
		if self.backing.is_none() {
			self.backing = if self.decorated {
				Some(draw::get_rect(fb, self.x - FRAME_BORDER, self.y - FRAME_TOP, self.bitmap.width + FRAME_BORDER * 2, self.bitmap.height + FRAME_TOP + FRAME_BORDER))
			} else {
				Some(draw::get_rect(fb, self.x, self.y, self.bitmap.width, self.bitmap.height))
			}
		}
		draw::map(&self.bitmap, fb, self.x, self.y);
		if self.decorated {
			self.decorate(fb);
		}
	}

	pub fn unmap(&mut self, fb: &mut BitMap) {
		if let Some(backing) = &self.backing {
			if self.decorated {
				draw::map(&backing, fb, self.x - FRAME_BORDER, self.y - FRAME_TOP);
			} else {
				draw::map(&backing, fb, self.x, self.y);
			}
			self.backing = None;
		}
	}

	pub fn decorate(&self, fb: &mut BitMap) {
		// border
		draw::draw_rect_stroke(
			self.bitmap.width + 2 * FRAME_BORDER,
			self.bitmap.height + FRAME_BORDER + FRAME_TOP,
			self.x - FRAME_BORDER,
			self.y - FRAME_TOP,
			Pixel::new(0xff, 0x00, 0x00),
			FRAME_BORDER,
			fb
		);
		// top
		draw::draw_rect(
			self.bitmap.width + 2 * FRAME_BORDER,
			FRAME_TOP,
			self.x - FRAME_BORDER,
			self.y - FRAME_TOP,
			Pixel::new(0xff, 0x00, 0x00),
			fb,
		);
	}
}


pub(crate) fn create_root(v_info: &fb::var_screeninfo) -> Window {
	let mut win = Window::create(v_info.xres, v_info.yres, 0, 0);
	win.bitmap.pxs = vec![Pixel::new(0xff, 0xff, 0xff); (v_info.xres * v_info.yres) as usize].into_boxed_slice();
	win.decorated = false;
	win
}
