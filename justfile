ci: test build doc clippy fmt forbidden-words

build:
  cargo build --all

test:
  cargo test --all -- --test-threads=1 --nocapture
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
