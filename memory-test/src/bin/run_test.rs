use anyhow::Result;
use cradle::prelude::*;
use std::process::{Command, Stdio};

fn from_mb(mega_bytes: usize) -> usize {
    mega_bytes * 2_usize.pow(20)
}

fn main() -> Result<()> {
    Split("cargo build --release").run_unit();
    let bytes = from_mb(64);
    let memory_consumption = measure_memory_consumption(bytes)?;
    let allowed_memory_consumption = from_mb(70); // should be 16
    assert!(
        memory_consumption < allowed_memory_consumption,
        "Maximum resident set size: {}, allowed upper limit: {}",
        memory_consumption,
        allowed_memory_consumption
    );
    Ok(())
}

fn measure_memory_consumption(bytes: usize) -> Result<usize> {
    let output = Command::new("/usr/bin/time")
        .arg("-v")
        .arg("./target/release/cradle_user")
        .arg(bytes.to_string())
        .stdout(Stdio::null())
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    eprintln!("{}", stderr);
    if !output.status.success() {
        panic!("running 'cradle_user' failed");
    }
    let memory_size_prefix = "Maximum resident set size (kbytes): ";
    let kilo_bytes: usize = strip_prefix(
        stderr
            .lines()
            .map(|x| x.trim())
            .find(|line| line.starts_with(memory_size_prefix))
            .unwrap(),
        memory_size_prefix,
    )
    .parse()?;
    let bytes = kilo_bytes * 1024;
    Ok(bytes)
}

fn strip_prefix<'a>(string: &'a str, prefix: &'a str) -> &'a str {
    #[allow(clippy::manual_strip)]
    if string.starts_with(prefix) {
        &string[prefix.len()..]
    } else {
        panic!("{} doesn't start with {}", string, prefix);
    }
}
