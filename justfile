_default:
  @just --choose

check:
  @export RUSTFLAGS="-D warnings" && \
  cargo check && \
  cargo clippy && \
  cargo fmt -- --check && \
  cargo test --all --no-fail-fast && \
  cargo build

watch-test isolate="":
  @watchexec --restart --clear --watch src cargo test {{isolate}}

test-new isolate="":
  @cd parser && watchexec --restart --clear --watch . cargo test {{isolate}}

