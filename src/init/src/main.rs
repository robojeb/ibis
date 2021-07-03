use std::{fs::File, io::Read};

fn main() {
    let mut logo_file = File::open("/logo.txt").unwrap();
    let mut buffer = String::new();

    logo_file.read_to_string(&mut buffer).unwrap();

    println!("Hello, Ibis!\n{}", buffer);

    loop {
        // Infinitely respawn shells
        let mut child = std::process::Command::new("/ibish").spawn().unwrap();
        child.wait();
    }
}
