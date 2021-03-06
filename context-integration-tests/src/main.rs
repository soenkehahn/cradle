fn main() {
    #[cfg(unix)]
    {
        {
            use cradle::prelude::*;
            run!(
                LogCommand,
                %"cargo build --bin test_executables_helper --features test_executables",
            );
        }

        use cradle::*;
        use executable_path::executable_path;
        use gag::BufferRedirect;
        use std::io::{self, Read};

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

        {
            assert_eq!(
                with_gag(BufferRedirect::stdout, || run_output!(%"echo foo")),
                "foo\n"
            );
        }

        {
            assert_eq!(
                with_gag(BufferRedirect::stderr, || run_output!(
                    executable_path("test_executables_helper").to_str().unwrap(),
                    "write to stderr"
                )),
                "foo\n"
            );
        }
        eprintln!("context integration tests: SUCCESS")
    }
}
