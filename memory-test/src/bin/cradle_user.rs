use cradle::prelude::*;

fn main() {
    let mut args = std::env::args();
    let bytes: usize = args.nth(1).unwrap().parse().unwrap();
    eprintln!("consuming {} KiB", bytes / 2_usize.pow(10));
    cmd_unit!("./target/release/produce_bytes", bytes.to_string());
}
