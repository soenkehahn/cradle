use anyhow::Result;
use std::io::{stdout, Write};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let mut bytes: usize = args.nth(1).unwrap().parse()?;
    eprintln!("producing {} KiB", bytes / 2_usize.pow(10));
    let buffer = &[b'x'; 1024];
    let mut stdout = stdout();
    while bytes > 0 {
        stdout.write_all(&buffer[..bytes])?;
        bytes -= bytes;
    }
    stdout.flush()?;
    Ok(())
}
