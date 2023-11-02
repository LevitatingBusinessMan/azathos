#[macro_export]
macro_rules! red {
	($str:expr) => {
		format!("\x1b[31m{}\x1b[39m", $str)
	}
}

#[macro_export]
macro_rules! green {
	($str:expr) => {
		format!("\x1b[32m{}\x1b[39m", $str)
	}
}

#[macro_export]
macro_rules! blue {
	($str:expr) => {
		format!("\x1b[34m{}\x1b[39m", $str)
	}
}

#[macro_export]
macro_rules! yellow {
	($str:expr) => {
		format!("\x1b[33m{}\x1b[39m", $str)
	}
}
