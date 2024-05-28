fn main() {
    println!("{}", std::env::args().skip(1).collect::<Vec<_>>().join(" "))
}
