use executable_path::executable_path;
use gag::BufferRedirect;
use std::io::{self, Read};
use stir::cmd;

fn with_gag<F>(mk_buf: fn() -> io::Result<BufferRedirect>, f: F) -> String
where
    F: FnOnce(),
{
    let mut buf = mk_buf().unwrap();
    f();
    let mut output = String::new();
    buf.read_to_string(&mut output).unwrap();
    output
}

fn main() {
    {
        assert_eq!(
            with_gag(BufferRedirect::stdout, || {
                let () = cmd!("echo foo");
            }),
            "foo\n"
        );
    }
    {
        assert_eq!(
            with_gag(BufferRedirect::stderr, || {
                let () = cmd!(
                    executable_path("stir_test_helper").to_str().unwrap(),
                    vec!["write to stderr"]
                );
            }),
            "foo\n"
        );
    }
    eprintln!("context integration tests: SUCCESS")
}
