use cradle::prelude::*;

fn main() {
    let args = std::env::args();
    let bytes: usize = args.skip(1).next().unwrap().parse().unwrap();
    eprintln!("consuming {} kB", bytes / 2_usize.pow(10));
    cmd_unit!("./target/release/produce_bytes", bytes.to_string());
}
