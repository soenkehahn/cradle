ci: test build doc clippy fmt forbidden-words

build:
  cargo build --all --features="build_test_helper"

test: build
  cargo test --all -- --test-threads=1
  rm 'filename with spaces'

doc:
  cargo doc --all

clippy:
  cargo clippy --all

fmt:
  cargo fmt --all -- --check

forbidden-words:
  ! grep -rni \
    'dbg!\|fixme\|todo' \
    src
  @echo No forbidden words found
