# A POSIX-Compliant Shell (CLI) Implementation in Rust

[![license](https://img.shields.io/badge/License-MIT-blue.svg?style=flat)](LICENSE)

# Supported Builtin Commands

- [cd](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cd.html) - change the working directory
- [echo [string...]](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/echo.html) - write arguments to standard
  output
- [exit [n]](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#exit) - cause the shell to exit
- [pwd](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pwd.html) - return working directory name
- [type [type name...]](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/type.html) - write a description of
  command type

# Notes

- [Bash](https://www.gnu.org/software/bash/) is used as a reference.
- Supports running external programs with arguments.
    - External programs are located using the [PATH](https://en.wikipedia.org/wiki/PATH_(variable)) environment
      variable.
- Supports [single quotes](https://www.gnu.org/software/bash/manual/bash.html#Single-Quotes).
- Supports [double quotes](https://www.gnu.org/software/bash/manual/bash.html#Double-Quotes).
- Supports [escape character](https://www.gnu.org/software/bash/manual/bash.html#Escape-Character) outside quotes.
- Supports [redirecting output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output).
- Some variants of some features may not be supported, i.e., not all features are fully-supported, with all their
  details and options.

# Running the Program

```shell
$ ./run.sh
```
