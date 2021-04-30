ci: test build doc clippy fmt

build:
  cargo build --all

test:
  cargo test --all -- --test-threads=1 --nocapture

doc:
  cargo doc --all

clippy:
  cargo clippy --all

fmt:
  cargo fmt --all -- --check
