use anyhow::Result;
use std::io::{stdout, Write};

fn main() -> Result<()> {
    let args = std::env::args();
    let bytes: usize = args.skip(1).next().unwrap().parse()?;
    eprintln!("producing {} kB", bytes / 2_usize.pow(10));
    let bytes = vec![b'x'; bytes];
    let mut stdout = stdout();
    stdout.write_all(&bytes)?;
    stdout.flush()?;
    Ok(())
}
