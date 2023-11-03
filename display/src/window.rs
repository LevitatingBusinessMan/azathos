use crate::{FrameBuffer, Pixel, draw};

pub(crate) struct Window {
	width: u32,
	height: u32,
	x: u32,
	y: u32,	
}

impl Window {
	pub fn create(width: u32, height: u32, loc: Option<(u32, u32)>) -> Self {
		let loc = loc.unwrap_or((0,0));
		Self {
			width,
			height,
			x: loc.0,
			y: loc.1,
		}
	}
	pub fn draw(&self, fb: &mut FrameBuffer, v_info: &fb::var_screeninfo) {
		draw::draw_rect_border(self.width, self.height, self.x, self.y, Pixel::new(0x00, 0xff, 0x00, 0x00), 1, fb, v_info);
		draw::draw_rect(self.width, self.height, self.x, self.y, Pixel::new(0x00, 0x00, 0x00, 0x00), fb, v_info);
	}
}
