use std::io::Write;

fn main() {
    std::io::stdout().write_all(&[0x80]).unwrap();
}
