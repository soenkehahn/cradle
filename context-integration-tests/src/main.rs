fn main() {
    #[cfg(unix)]
    {
        use cradle::*;
        use gag::BufferRedirect;
        use std::{
            fs,
            io::{self, Read},
        };
        use tempfile::TempDir;
        use unindent::Unindent;

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
            let temp_dir = TempDir::new().unwrap();
            let script = temp_dir.path().join("script.py");
            fs::write(
                &script,
                "
                    import sys
                    print('foo', file=sys.stderr)
                "
                .unindent(),
            )
            .unwrap();
            assert_eq!(
                with_gag(BufferRedirect::stderr, || run_output!("python3", script)),
                "foo\n"
            );
        }
        eprintln!("context integration tests: SUCCESS")
    }
}
