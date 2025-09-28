#!/bin/sh

# Exit early if any commands fail.
set -e

# We need to set DEBUG before we build the project,
# as it is only considered during compile time, and not during run time.
export DEBUG=true

(
  cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory.
  cargo build --release --target-dir=/tmp/build-shell --manifest-path Cargo.toml
)

exec /tmp/build-shell/release/posix-shell "$@"
