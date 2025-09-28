# A POSIX-Compliant Shell (CLI) Implementation in Rust

[![license](https://img.shields.io/badge/License-MIT-blue.svg?style=flat)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/posix-shell.svg)](https://crates.io/crates/posix-shell)
[![docs.rs](https://docs.rs/posix-shell/badge.svg)](https://docs.rs/posix-shell/)
[![CI](https://github.com/ivanbgd/posix-shell-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/ivanbgd/posix-shell-rust/actions/workflows/ci.yml)
[![Security audit](https://github.com/ivanbgd/posix-shell-rust/actions/workflows/audit.yml/badge.svg)](https://github.com/ivanbgd/posix-shell-rust/actions/workflows/audit.yml)

# Supported Builtin Commands

- [cd](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cd.html) - change the working directory
- [echo [string...]](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/echo.html) - write arguments to standard
  output
- [exit [n]](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#exit) - cause the shell to exit
- [pwd](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pwd.html) - return working directory name
- [type [type name...]](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/type.html) - write a description of
  command type

# Notes

- Mostly [bash](https://www.gnu.org/software/bash/) is used as a reference, but not everything is in accordance
  with `bash`. Some things are in accordance with the `zsh` - for example, multiple redirection.
- Supports running external programs with arguments.
    - External programs are located using the [PATH](https://en.wikipedia.org/wiki/PATH_(variable)) environment
      variable.
- Supports [single quotes](https://www.gnu.org/software/bash/manual/bash.html#Single-Quotes).
- Supports [double quotes](https://www.gnu.org/software/bash/manual/bash.html#Double-Quotes).
- Supports [escape character](https://www.gnu.org/software/bash/manual/bash.html#Escape-Character) outside quotes.
- Supports [redirecting output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output).
- Supports
  [appending redirected output](https://www.gnu.org/software/bash/manual/bash.html#Appending-Redirected-Output).
- Supports shell-specific `&>word` and shell-specific `>&word`, which redirect both `stdout` and `stderr` to the file
  whose name is the expansion of `word`.
- Supports multiple redirections.

# Running the Program

```shell
$ ./run.sh
```

# Building and Running the Program with Debug Output

The program supports debugging output, which can be enabled by setting
the environment variable `DEBUG` to the value `true`.

It can be set outside the program, in the user shell, or inside the `run.sh` shell script.

This is only considered during **compile time**, and **not** during run time.

```shell
$ DEBUG=true ./run.sh
```

```shell
export DEBUG=true
$ ./run.sh
```

# Testing

- Unit-test with `cargo test`.
- End-to-end-test with `./test.sh`.
