use std::io::Write;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    match args.next().unwrap().as_str() {
        "invalid utf-8 stdout" => std::io::stdout().write_all(&[0x80]).unwrap(),
        "exit code 42" => std::process::exit(42),
        arg => panic!("stir_test_helper: invalid arg: {}", arg),
    }
}
