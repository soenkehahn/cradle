ci: test build doc clippy fmt forbidden-words

build:
  cargo build --all --features="build_test_helper"

test pattern="": build
  cargo test --all -- --test-threads=1 {{ pattern }}
  rm -f 'filename with spaces' foo

doc:
  cargo doc --all

clippy:
  cargo clippy --all

fmt:
  cargo fmt --all -- --check

forbidden-words:
  ! grep -rni \
    'dbg!\|fixme\|todo\|ignore' \
    src
  @echo No forbidden words found
