ci: test build doc clippy fmt integration forbidden-words

build:
  cargo build --all --features="test_executables"

test pattern="": build
  cargo test --all -- --test-threads=1 {{ pattern }}
  rm -f 'filename with spaces' foo

integration: build
  cargo run --features "test_executables" --bin context_integration_tests

doc:
  cargo doc --all

clippy:
  cargo clippy --all --features="test_executables"

fmt:
  cargo fmt --all -- --check

forbidden-words:
  ! grep -rni \
    'dbg!\|fixme\|todo\|ignore' \
    src
  @echo No forbidden words found
