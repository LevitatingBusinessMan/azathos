mod command {
	pub struct CreateWindow {
		width: u32,
		height: u32,
	}
	
	pub enum Command {
		CreateWindow(CreateWindow)
	}
}
