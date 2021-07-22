use std::{
    env::{current_dir, set_current_dir},
    fs,
};
use tempfile::TempDir;

pub(crate) fn in_temporary_directory<F>(f: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let temp_dir = TempDir::new().unwrap();
    let original_working_directory = current_dir().unwrap();
    set_current_dir(&temp_dir).unwrap();
    let result = std::panic::catch_unwind(|| {
        f();
    });
    set_current_dir(original_working_directory).unwrap();
    result.unwrap();
}

pub(crate) fn with_script<F>(script: &str, test: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    in_temporary_directory(|| {
        let prefix = vec!["#!/usr/bin/env bash", "set -euo pipefail"].join("\n");
        fs::write("test-script.sh", format!("{}\n\n{}", prefix, script)).unwrap();
        #[cfg(unix)]
        crate::cmd_unit!(%"chmod +x test-script.sh");
        test();
    });
}
