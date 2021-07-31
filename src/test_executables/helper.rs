use std::io::{self, Write};

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    match args.next().unwrap().as_str() {
        "invalid utf-8 stdout" => io::stdout().write_all(&[0x80]).unwrap(),
        "invalid utf-8 stderr" => io::stderr().write_all(&[0x80]).unwrap(),
        "output foo and exit with 42" => {
            println!("foo");
            std::process::exit(42)
        }
        arg => panic!("cradle_test_helper: invalid arg: {}", arg),
    }
}
