This is an mp4 track metadata parser.

[![Latest crate version](https://meritbadge.herokuapp.com/mp4parse)](https://crates.io/crates/mp4parse)
[![Travis build status](https://travis-ci.org/mozilla/mp4parse-rust.svg)](https://travis-ci.org/mozilla/mp4parse-rust)

Our primary interest is writing a pure-rust replacement for the
track metadata parser needed by Firefox.

To enable it on Mac and Linux builds of Firefox, add `ac_add_options --enable-rust` to your `.mozconfig`.
