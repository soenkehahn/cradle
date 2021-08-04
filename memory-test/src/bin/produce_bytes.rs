use anyhow::Result;
use std::io::{stdout, Write};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let mut bytes: usize = args.nth(1).unwrap().parse()?;
    eprintln!("producing {} kB", bytes / 2_usize.pow(10));
    let buffer = &[b'x'; 1024];
    let mut stdout = stdout();
    while bytes > 0 {
        if bytes >= buffer.len() {
            stdout.write_all(buffer)?;
            bytes -= buffer.len();
        } else {
            stdout.write_all(&[b'x'])?;
            bytes -= 1;
        }
    }
    stdout.flush()?;
    Ok(())
}
