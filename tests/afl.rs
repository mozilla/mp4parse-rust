// Regression tests from American Fuzzy Lop test cases.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

extern crate mp4parse;

use std::io::Cursor;

/// https://github.com/mozilla/mp4parse-rust/issues/2
#[test]
fn fuzz_2() {
    let mut c = Cursor::new(b"\x00\x00\x00\x04\xa6\x00\x04\xa6".to_vec());
    let mut context = mp4parse::MediaContext::new();
    let _ = mp4parse::read_box(&mut c, &mut context);
}

/// https://github.com/mozilla/mp4parse-rust/issues/4
#[test]
fn fuzz_4() {
    let mut c = Cursor::new(b"\x00\x00\x00\x01\x30\x30\x30\x30\x00\x00\x00\x00\x00\x00\x00\x00".to_vec());
    let mut context = mp4parse::MediaContext::new();
    let _ = mp4parse::read_box(&mut c, &mut context);
}

/// https://github.com/mozilla/mp4parse-rust/issues/5
#[test]
fn fuzz_5() {
    let mut c =
    Cursor::new(b"\x30\x30\x30\x30\x66\x74\x79\x70\x30\x30\x30\x30\x30\x30\x30\x30".to_vec());
    let mut context = mp4parse::MediaContext::new();
    let _ = mp4parse::read_box(&mut c, &mut context);
}
