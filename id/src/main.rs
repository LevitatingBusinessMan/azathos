use clap::Parser;

#[derive(Parser)]
struct Args {

}

fn main() {
    let args = Args::parse();
    unsafe {
        let uid = libc::getuid();
        let gid = libc::getgid();
        println!("uid: {uid}\nguid: {gid}");
    }
}
