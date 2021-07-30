use crate::{config::Config, input::Input};
use std::{fs, path::PathBuf};
use tempfile::TempDir;
use unindent::Unindent;

pub struct TestScript {
    temp_dir: TempDir,
}

impl TestScript {
    pub fn new(code: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let result = Self { temp_dir };
        fs::write(result.script_path(), code.unindent()).unwrap();
        result
    }

    fn script_path(&self) -> PathBuf {
        self.temp_dir.path().join("test-script.py")
    }
}

impl Input for &TestScript {
    fn configure(self, config: &mut Config) {
        ("python3", self.script_path()).configure(config)
    }
}
