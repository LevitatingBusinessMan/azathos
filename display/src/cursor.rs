use std::path::{PathBuf, Path};

use crate::{window::Window, ppf, draw::draw_rect};

pub(crate) fn cursor() -> Window {
	let mut cursor = Window::create(10, 10, 0, 0);
	cursor.decorated = false;
	let img = ppf::load(Path::new("/share/display/cursors/cursor.ppf").to_owned()).unwrap();
	cursor.bitmap = img;
	cursor
}
