use std::{rc::Rc, cell::RefCell, sync::{Mutex, Arc, RwLock}};
use slab_tree::{Tree, NodeId, NodeMut};

use crate::{FrameBuffer, Pixel, draw, BitMap};

type WindowId = NodeId;

pub struct Window {
	pub x: u32,
	pub y: u32,
	pub bitmap: BitMap,
}

impl Window {
	// pub fn create(width: u32, height: u32, loc: Option<(u32, u32)>, tree: Rc<Vec<Window>>, id: WindowId) -> Self {
	// 	let loc = loc.unwrap_or((0,0));
	// 	Self {
	// 		x: loc.0,
	// 		y: loc.1,
	// 		parent: None,
	// 		children: vec![],
	// 		tree,
	// 		id,
	// 		bitmap: BitMap { height, width, pxs: vec![Pixel::new(0, 0, 0, 0)].into_boxed_slice() },
	// 	}
	// }
	pub fn map(&mut self, fb: &mut BitMap) {
		draw::map(&self.bitmap, fb, self.x, self.y)
	}
	pub fn decorate(&self, fb: &mut FrameBuffer, v_info: &fb::var_screeninfo) {
		draw::draw_rect_border(self.bitmap.height, self.bitmap.height, self.x, self.y, Pixel::new(0x00, 0xff, 0x00, 0x00), 1, fb, v_info);
	}
}

// pub fn draw(&self, fb: &mut FrameBuffer, v_info: &fb::var_screeninfo) {
// 	draw::draw_rect_border(self.width, self.height, self.x, self.y, Pixel::new(0x00, 0xff, 0x00, 0x00), 1, fb, v_info);
// 	draw::draw_rect(self.width, self.height, self.x, self.y, Pixel::new(0x00, 0x00, 0x00, 0x00), fb, v_info);
// }
