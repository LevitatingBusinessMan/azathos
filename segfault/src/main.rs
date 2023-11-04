//! For when you just really want to segfault

fn main() {
   unsafe { *(0xdead10cc_usize as * mut u32) = 0xdead };
}
