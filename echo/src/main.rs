fn main() {
    println!("{:?}", std::env::args().collect::<Vec<_>>().join(" "))
}
