// C API for mp4parse module.
// Parses ISO Base Media Format aka video/mp4 streams.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std;
use std::io::Cursor;
use byteorder;

// Symbols we need from our rust api.
use MediaContext;
use read_box;

/// Allocate an opaque rust-side parser context.
#[no_mangle]
pub unsafe extern "C" fn mp4parse_new() -> *mut MediaContext {
    std::mem::transmute(Box::new(MediaContext::new()))
}

/// Free an rust-side parser context.
#[no_mangle]
pub unsafe extern "C" fn mp4parse_free(context: *mut MediaContext) {
    assert!(!context.is_null());
    let _: Box<MediaContext> = std::mem::transmute(context);
}

/// Feed a buffer to read_box() it, returning the number of detected tracks.
#[no_mangle]
pub extern "C" fn mp4parse_read(context: *mut MediaContext, buffer: *const u8, size: usize) -> i32 {
    // Validate arguments from C.
    if context.is_null() {
        return -1;
    }
    if buffer.is_null() || size < 8 {
        return -1;
    }

    let mut context: &mut MediaContext = unsafe { &mut *context };

    // Wrap the buffer we've been give in a slice.
    let b = unsafe { std::slice::from_raw_parts(buffer, size) };
    let mut c = Cursor::new(b);

    // Parse in a subthread to catch any panics.
    let task = std::thread::spawn(move || {
        loop {
            match read_box(&mut c, &mut context) {
                Ok(_) => {},
                Err(byteorder::Error::UnexpectedEOF) => { break },
                Err(e) => { panic!(e); },
            }
        }
        // Make sure the track count fits in an i32 so we can use
        // negative values for failure.
        assert!(context.tracks.len() < std::i32::MAX as usize);
        context.tracks.len() as i32
    });
    task.join().unwrap_or(-1)
}
