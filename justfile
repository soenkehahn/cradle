ci: test build doc clippy fmt context-integration-tests run-examples forbidden-words render-readme-check

build:
  cargo build --all-targets --all-features

test +pattern="": build
  cargo test --all {{ pattern }}

test-lib-fast +pattern="":
  cargo test --lib {{ pattern }}

context-integration-tests: build
  cargo run --features "test_executables" --bin context_integration_tests

doc +args="":
  cargo doc --all {{args}}

clippy:
  cargo clippy --all-targets --all-features

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
    'dbg!\|fixme\|todo\|ignore' \
    src tests examples
  @echo No forbidden words found
