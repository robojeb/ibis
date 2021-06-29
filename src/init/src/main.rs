use std::{
    io::Read,
    fs::File,
};

fn main() {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();

    logo_file.read_to_string(&mut buffer).unwrap();

    println!("Hello, Ibis!\n{}", buffer);
    loop {}
}
