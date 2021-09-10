[![ci status badge](https://github.com/soenkehahn/cradle/actions/workflows/ci.yaml/badge.svg)](https://github.com/soenkehahn/cradle/actions?query=branch%3Amaster)
[![crates.io](https://img.shields.io/crates/v/cradle.svg)](https://crates.io/crates/cradle)
[![docs](https://docs.rs/cradle/badge.svg)](https://docs.rs/cradle)

`cradle` is a library for executing child processes.
It provides a more convenient interface than
[std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html).
Here's an example:

``` rust
<?php include("./examples/readme.rs") ?>
```

For comprehensive documentation, head over to
[docs.rs/cradle](https://docs.rs/cradle/latest/cradle/).

## Design Goals

`cradle` is meant to make it as easy as possible to run child processes,
while making it very hard to use incorrectly.
As such it provides an interface that is very concise, yet flexible,
but tries to avoid any behavior that would be unexpected or surprising.

`cradle` decidedly does _not_ try to emulate any syntax of `bash` or other shells,
like piping (`|`) or shell expansion (e.g. globs, like `*`).
Instead, it is aiming to be a convenience wrapper around your
operating system's interface for running child processes.

## MSRV
The minimal supported rust version is `0.41`.
