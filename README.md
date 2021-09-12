[![ci status badge](https://github.com/soenkehahn/cradle/actions/workflows/ci.yaml/badge.svg)](https://github.com/soenkehahn/cradle/actions?query=branch%3Amaster)
[![crates.io](https://img.shields.io/crates/v/cradle.svg)](https://crates.io/crates/cradle)
[![docs](https://docs.rs/cradle/badge.svg)](https://docs.rs/cradle)

`cradle` is a library for executing child processes.
It provides a more convenient interface than
[std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html).
Here's an example:

``` rust
use cradle::prelude::*;

fn main() {
    // output git version
    run!(%"git --version");
    // output configured git user
    let (StdoutTrimmed(git_user), Status(status)) = run_output!(%"git config --get user.name");
    if status.success() {
        eprintln!("git user: {}", git_user);
    } else {
        eprintln!("git user not configured");
    }
}
```

For comprehensive documentation, head over to
[docs.rs/cradle](https://docs.rs/cradle/latest/cradle/).

## Design Goals

`cradle` is meant to make it as easy as possible to run child processes,
while making it hard to use incorrectly.
As such it provides an interface that is concise and flexible, and tries to avoid surprising behavior.

`cradle` does not try to emulate the syntax or functionality of `bash` or other shells,
such as pipes (`|`), globs (`*`), or other string expansion.
Instead, it aims to be a convenient wrapper around the
operating system's interface for running child processes.

## MSRV
The minimal supported rust version is `0.41`.
