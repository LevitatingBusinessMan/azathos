use std::{rc::Rc, cell::RefCell, sync::{Mutex, Arc, RwLock}};
use slab_tree::{Tree, NodeId, NodeMut};

use crate::{FrameBuffer, Pixel, draw, BitMap};

type WindowId = NodeId;

pub struct Window {
	pub id: u64,
	/// Absolute X coordinate, regardless of decorations
	pub x: u32,
	/// Absolute Y coordinate, regardless of decorations
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

   /// x coordinate of internal bitmap
   /// (thus excluding the decoration).
   pub fn ix(&self) -> u32 {
	if self.decorated {
		self.x + FRAME_BORDER
	} else {
		self.x
	}
   }

	/// y coordinate of internal bitmap
   /// (thus excluding the decoration).
   pub fn iy(&self) -> u32 {
		if self.decorated {
			self.y + FRAME_TOP
		} else {
			self.y
		}
   }

   /// Total width, including decorations.
   pub fn twidth(&self) -> u32 {
		if self.decorated {
			self.bitmap.width + FRAME_BORDER * 2
		} else {
			self.bitmap.width
		}
   }

   /// Total height, including decorations.
   pub fn theight(&self) -> u32 {
	if self.decorated {
		self.bitmap.height + FRAME_BORDER + FRAME_TOP
	} else {
		self.bitmap.height
	}
   }

   	/// Map this onto the framebuffer (or another bitmap...)
	pub fn map(&mut self, fb: &mut BitMap) {
		if self.backing.is_none() {
			self.backing = Some(draw::get_rect(fb, self.x, self.y, self.twidth(), self.theight()))
		}
		draw::map(&self.bitmap, fb, self.ix(), self.iy());
		if self.decorated {
			self.decorate(fb);
		}
	}

	/// A copy of [Window::map] but with pixel blending
	pub fn map_alpha(&mut self, fb: &mut BitMap) {
		if self.backing.is_none() {
			self.backing = Some(draw::get_rect(fb, self.x, self.y, self.twidth(), self.theight()))
		}
		draw::map_alpha(&self.bitmap, fb, self.ix(), self.iy());
		if self.decorated {
			self.decorate(fb);
		}
	}

	pub fn unmap(&mut self, fb: &mut BitMap) {
		if let Some(backing) = &self.backing {
			draw::map(&backing, fb, self.x, self.y);
			self.backing = None;
		}
	}

	pub fn decorate(&self, fb: &mut BitMap) {
		// border
		draw::draw_rect_stroke(
			self.twidth(),
			self.theight(),
			self.x,
			self.y,
			Pixel::new(0xff, 0x00, 0x00),
			FRAME_BORDER,
			fb
		);
		// top
		draw::draw_rect(
			self.twidth(),
			FRAME_TOP,
			self.x,
			self.y,
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
