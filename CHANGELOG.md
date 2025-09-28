# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Pipelines
- Autocompletion
- History

## [0.1.0] - 2025-06-19

This is the initial fully-functioning version of the library and the application.

### Added

- Library crate
- Binary (executable) crate, which uses the library
- Basic functionality:
    - Builtins: `cd`, `echo`, `exit`, `pwd`, `type`
    - Running external programs with arguments using the `PATH` environment variable
    - Quoting: single and double quotes, escape character
    - Redirection: redirecting output, appending redirected output, multiple redirections - `stdout` and `stderr`
- `README.md`
- `LICENSE` ("MIT")
- `CHANGELOG.md`
- GitHub action `ci.yml`
- GitHub action `release.yml`
