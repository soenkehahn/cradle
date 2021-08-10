use anyhow::Result;
use std::process::{Command, Stdio};

fn from_mib(mebibytes: usize) -> usize {
    mebibytes * 2_usize.pow(20)
}

fn main() -> Result<()> {
    test("stdout")?;
    test("stderr")?;
    Ok(())
}

fn test(stream_type: &str) -> Result<()> {
    let bytes = from_mib(64);
    let memory_consumption = measure_memory_consumption(stream_type, bytes)?;
    let allowed_memory_consumption = from_mib(16);
    assert!(
        memory_consumption < allowed_memory_consumption,
        "stream type: {}, Maximum resident set size: {}, allowed upper limit: {}",
        stream_type,
        memory_consumption,
        allowed_memory_consumption
    );
    Ok(())
}

fn measure_memory_consumption(stream_type: &str, bytes: usize) -> Result<usize> {
    let output = Command::new("/usr/bin/time")
        .arg("-v")
        .arg("./target/release/cradle_user")
        .arg(stream_type)
        .arg(bytes.to_string())
        .stdout(Stdio::null())
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    if !output.status.success() {
        eprintln!("{}", stderr);
        panic!("running 'cradle_user' failed");
    }
    let memory_size_prefix = "Maximum resident set size (kbytes): ";
    let kibibytes: usize = strip_prefix(
        stderr
            .lines()
            .map(|line| line.trim())
            .find(|line| line.starts_with(memory_size_prefix))
            .unwrap(),
        memory_size_prefix,
    )
    .parse()?;
    let bytes = kibibytes * 1024;
    Ok(bytes)
}

#[rustversion::attr(since(1.48), allow(clippy::manual_strip))]
fn strip_prefix<'a>(string: &'a str, prefix: &'a str) -> &'a str {
    if string.starts_with(prefix) {
        &string[prefix.len()..]
    } else {
        panic!("{} doesn't start with {}", string, prefix);
    }
}
