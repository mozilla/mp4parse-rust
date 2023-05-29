This is an mp4 track metadata parser.

[![Latest crate version](https://img.shields.io/crates/v/mp4parse.svg)](https://crates.io/crates/mp4parse)
[![Build status](https://github.com/mozilla/mp4parse-rust/actions/workflows/build.yml/badge.svg)](https://github.com/mozilla/mp4parse-rust/actions)

Our primary interest is writing a pure-rust replacement for the
track metadata parser needed by Firefox.

[API documentation](https://docs.rs/mp4parse/)

# Project structure

`mp4parse` is a parser for ISO base media file format (mp4) written in rust.

`mp4parse-capi` is a C API that exposes the functionality of `mp4parse`. The C
API is intended to wrap the rust parser. As such, features should primarily
be implemented in the rust parser and exposed via the C API, rather than the C
API implementing features on its own.

## Tests

Test coverage comes from several sources:
- Conventional tests exist in `mp4parse/src/lib.rs` and
`mp4parse_capi/src/lib.rs` as well as under `mp4parse/tests` and
`mp4parse_capi/tests`. These tests can be run via `cargo test`.
- Examples are included under `mp4parse_capi/examples`. These programs should
continue to build and run after changes are made. Note, these programs are not
typically run by `cargo test`, so manual verification is required.

# Versioning

The master branch represents the last version released to crates.io plus any
development since that release.  Firefox will ship specific git revisions from
the master branch (refer to the `mp4parse_capi` dependency listed in
[toolkit/library/rust/shared/Cargo.toml](https://searchfox.org/mozilla-central/source/toolkit/library/rust/shared/Cargo.toml#15)
for the currently shipping revision).  When sufficient changes to merit a new
crates.io release have occurred, the version in Cargo.toml will be bumped and
tagged, and the new version published to crates.io.
