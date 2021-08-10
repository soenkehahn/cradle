use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let stream_type: String = args.nth(1).unwrap();
    let mut bytes: usize = args.next().unwrap().parse()?;
    eprintln!("writing {} KiB to {}", bytes / 2_usize.pow(10), stream_type);
    let buffer = &[b'x'; 1024];
    let mut stream: Box<dyn Write> = match stream_type.as_str() {
        "stdout" => Box::new(io::stdout()),
        "stderr" => Box::new(io::stderr()),
        _ => panic!("unknown stream type: {}", stream_type),
    };
    while bytes > 0 {
        let chunk_size = bytes.min(1024);
        stream.write_all(&buffer[..chunk_size])?;
        bytes -= chunk_size;
    }
    stream.flush()?;
    Ok(())
}
