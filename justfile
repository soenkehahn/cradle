ci: test build doc clippy fmt run-examples forbidden-words render-readme-check

build:
  cargo build --all-targets --all-features --workspace

test +pattern="":
  cargo test {{ pattern }}

test-lib-fast +pattern="":
  cargo test --lib {{ pattern }}

doc +args="":
  cargo doc --workspace {{args}}

clippy:
  cargo clippy --all-targets --all-features --workspace

fmt:
  cargo fmt --all -- --check

run-examples:
  cargo run --example readme

render-readme:
  php README.php > README.md

render-readme-check:
  #!/usr/bin/env bash
  diff <(php README.php) README.md

forbidden-words:
  ! grep -rni \
    'dbg!\|fixme\|todo\|#\[ignore\]' \
    src tests examples
  @echo No forbidden words found

all-rustc-versions *args="ci":
  #!/usr/bin/env bash
  set -eu

  export RUSTFLAGS="--deny warnings"

  # install yq with: pip3 install yq
  versions=$(cat .github/workflows/ci.yaml \
    | yq -r '.jobs.all.strategy.matrix.rust | sort | join(" ")' \
    ;)
  for RUSTUP_TOOLCHAIN in $versions
  do
    export RUSTUP_TOOLCHAIN
    cargo version
    just {{ args }}
  done
