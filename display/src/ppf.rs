//! The Pixel Perfect Format

use std::{io, path::{PathBuf}, fs};
use std::str;

use crate::{BitMap, Pixel};

/**
 * PIXEL PERFECT FORMAT
 * Starts with 'ppf' and a newline
 * Optional comment lines starting with #
 * Rows of 1's and 0's delimited with a space
 * Row length and amount correspond to the image dimensions
 * 
 * 1 = black
 * 0 = white
 * x = transparency
 */

pub fn load(path: PathBuf) -> io::Result<BitMap> {
	let data = fs::read(path)?;
	if !data.starts_with(&[b'p',b'p',b'f', b'\n']) {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected magic bytes"));
	}

	let mut pxs: Vec<Pixel> = vec![];
	let mut width = 0;
	let mut height = 0;
	for line in str::from_utf8(&data[4..])
	.map_err(
		|e| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8")
	)?.lines() {
		if line.starts_with('#') {
			continue
		}
		if width == 0 {
			width = line.split_ascii_whitespace().count() as u32;
		}
		for px in line.split_ascii_whitespace() {
			let px = match px {
				"1" => Pixel::new(0x00, 0x00, 0x00),
				"0" => Pixel::new(0xff, 0xff, 0xff),
				"x" => Pixel { blue: 0xff, green: 0xff, red: 0xff, _alpha: 0xff },
				_ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data found"))
			};
			pxs.push(px);
		}
		height += 1;
	}
	if pxs.len() != (height * width) as usize {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "Rows did not match"))
	} else {
		Ok(BitMap {
			width,
			height,
			pxs: pxs.into_boxed_slice(),
		})
	}
}
