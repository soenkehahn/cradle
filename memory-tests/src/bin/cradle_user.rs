use cradle::prelude::*;

fn main() {
    let mut args = std::env::args();
    let stream_type: String = args.nth(1).unwrap();
    let bytes: usize = args.next().unwrap().parse().unwrap();
    eprintln!("consuming {} KiB", bytes / 2_usize.pow(10));
    cmd_unit!(
        "./target/release/produce_bytes",
        stream_type,
        bytes.to_string()
    );
}
