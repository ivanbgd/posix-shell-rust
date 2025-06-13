#!/bin/sh

set -e # Exit early if any commands fail

export DEBUG=true

(
  cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory
  cargo build --release --target-dir=/tmp/build-shell --manifest-path Cargo.toml
)

exec /tmp/build-shell/release/posix-shell "$@"
