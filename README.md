This is an mp4 track metadata parser.

[![Latest crate version](https://img.shields.io/crates/v/mp4parse.svg)](https://crates.io/crates/mp4parse)
[![Build status](https://github.com/mozilla/mp4parse-rust/actions/workflows/build/badge.svg)](https://github.com/mozilla/mp4parse-rust/actions)

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
