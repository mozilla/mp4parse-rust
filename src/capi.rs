//! C API for mp4parse module.
//!
//! Parses ISO Base Media Format aka video/mp4 streams.
//!
//! # Examples
//!
//! ```rust
//! extern crate mp4parse;
//!
//! // Minimal valid mp4 containing no tracks.
//! let data = b"\0\0\0\x08moov";
//!
//! let context = mp4parse::mp4parse_new();
//! unsafe {
//!     let rv = mp4parse::mp4parse_read(context, data.as_ptr(), data.len());
//!     assert_eq!(rv, mp4parse::mp4parse_error::MP4PARSE_OK);
//!     mp4parse::mp4parse_free(context);
//! }
//! ```

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std;
use std::io::Cursor;

// Symbols we need from our rust api.
use MediaContext;
use TrackType;
use read_mp4;
use Error;
use media_time_to_ms;
use track_time_to_ms;
use SampleEntry;

// rusty-cheddar's C enum generation doesn't namespace enum members by
// prefixing them, so we're forced to do it in our member names until
// https://github.com/Sean1708/rusty-cheddar/pull/35 is fixed.  Importing
// the members into the module namespace avoids doubling up on the
// namespacing on the Rust side.
use mp4parse_error::*;
use mp4parse_track_type::*;

#[repr(C)]
#[derive(PartialEq,Debug)]
pub enum mp4parse_error {
    MP4PARSE_OK = 0,
    MP4PARSE_ERROR_BADARG = 1,
    MP4PARSE_ERROR_INVALID = 2,
    MP4PARSE_ERROR_UNSUPPORTED = 3,
    MP4PARSE_ERROR_EOF = 4,
    MP4PARSE_ERROR_ASSERT = 5,
    MP4PARSE_ERROR_IO = 6,
}

#[repr(C)]
pub enum mp4parse_track_type {
    MP4PARSE_TRACK_TYPE_VIDEO = 0,
    MP4PARSE_TRACK_TYPE_AUDIO = 1,
}

#[repr(C)]
pub struct mp4parse_track_info {
    pub track_type: mp4parse_track_type,
    pub track_id: u32,
    pub duration: u64,
    pub media_time: i64, // wants to be u64? understand how elst adjustment works
    // TODO(kinetik): include crypto guff
}

#[repr(C)]
pub struct mp4parse_track_audio_info {
    pub channels: u16,
    pub bit_depth: u16,
    pub sample_rate: u32,
    // TODO(kinetik):
    // int32_t profile;
    // int32_t extended_profile; // check types
    // extra_data
    // codec_specific_config
}

#[repr(C)]
pub struct mp4parse_track_video_info {
    pub display_width: u32,
    pub display_height: u32,
    pub image_width: u16,
    pub image_height: u16,
    // TODO(kinetik):
    // extra_data
    // codec_specific_config
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct mp4parse_state(MediaContext);

// C API wrapper functions.

/// Allocate an opaque rust-side parser context.
#[no_mangle]
pub extern "C" fn mp4parse_new() -> *mut mp4parse_state {
    let context = Box::new(mp4parse_state(MediaContext::new()));
    Box::into_raw(context)
}

/// Free a rust-side parser context.
#[no_mangle]
pub unsafe extern "C" fn mp4parse_free(context: *mut mp4parse_state) {
    assert!(!context.is_null());
    let _ = Box::from_raw(context);
}

/// Feed a buffer through `read_mp4()` with the given rust-side
/// parser context, returning success or an error code.
///
/// This is safe to call with NULL arguments but will crash
/// if given invalid pointers, as is usual for C.
#[no_mangle]
pub unsafe extern "C" fn mp4parse_read(context: *mut mp4parse_state, buffer: *const u8, size: usize) -> mp4parse_error {
    // Validate arguments from C.
    if context.is_null() || buffer.is_null() || size < 8 {
        return MP4PARSE_ERROR_BADARG;
    }

    let mut context: &mut MediaContext = &mut (*context).0;

    // Wrap the buffer we've been give in a slice.
    let b = std::slice::from_raw_parts(buffer, size);
    let mut c = Cursor::new(b);

    let r = if cfg!(not(feature = "fuzz")) {
        // Parse in a subthread to catch any panics.
        let task = std::thread::spawn(move || read_mp4(&mut c, &mut context));
        // The task's JoinHandle will return an error result if the thread
        // panicked, and will wrap the closure's return'd result in an
        // Ok(..) otherwise, meaning we could see Ok(Err(Error::..))
        // here. So map thread failures back to an
        // mp4parse::Error::AssertCaught.
        task.join().unwrap_or(Err(Error::AssertCaught))
    } else {
        read_mp4(&mut c, &mut context)
    };
    match r {
        Ok(_) => MP4PARSE_OK,
        Err(Error::NoMoov) => MP4PARSE_ERROR_INVALID,
        Err(Error::InvalidData(_)) => MP4PARSE_ERROR_INVALID,
        Err(Error::Unsupported(_)) => MP4PARSE_ERROR_UNSUPPORTED,
        Err(Error::UnexpectedEOF) => MP4PARSE_ERROR_EOF,
        Err(Error::AssertCaught) => MP4PARSE_ERROR_ASSERT,
        Err(Error::Io(_)) => MP4PARSE_ERROR_IO,
    }
}

/// Return the number of tracks parsed by previous `read_mp4()` calls.
#[no_mangle]
pub unsafe extern "C" fn mp4parse_get_track_count(context: *const mp4parse_state) -> u32 {
    // Validate argument from C.
    assert!(!context.is_null());
    let context: &MediaContext = &(*context).0;

    // Make sure the track count fits in a u32.
    assert!(context.tracks.len() < u32::max_value() as usize);
    context.tracks.len() as u32
}

#[no_mangle]
pub unsafe extern "C" fn mp4parse_get_track_info(context: *mut mp4parse_state, track: u32, info: *mut mp4parse_track_info) -> mp4parse_error {
    if context.is_null() || info.is_null() {
        return MP4PARSE_ERROR_BADARG;
    }

    let context: &mut MediaContext = &mut (*context).0;
    let track_index: usize = track as usize;
    let info: &mut mp4parse_track_info = &mut *info;

    if track_index >= context.tracks.len() {
        return MP4PARSE_ERROR_BADARG;
    }

    info.track_type = match context.tracks[track_index].track_type {
        TrackType::Video => MP4PARSE_TRACK_TYPE_VIDEO,
        TrackType::Audio => MP4PARSE_TRACK_TYPE_AUDIO,
        TrackType::Unknown => return MP4PARSE_ERROR_UNSUPPORTED,
    };

    // Maybe context & track should just have a single simple is_valid() instead?
    if context.timescale.is_none() ||
       context.tracks[track_index].timescale.is_none() ||
       context.tracks[track_index].duration.is_none() ||
       context.tracks[track_index].track_id.is_none() {
        return MP4PARSE_ERROR_INVALID;
    }

    let track = &context.tracks[track_index];
    info.media_time = track.media_time.map_or(0, |media_time| {
        track_time_to_ms(media_time, track.timescale.unwrap()) as i64
    }) - track.empty_duration.map_or(0, |empty_duration| {
        media_time_to_ms(empty_duration, context.timescale.unwrap()) as i64
    });
    info.duration = track_time_to_ms(track.duration.unwrap(), track.timescale.unwrap());
    info.track_id = track.track_id.unwrap();

    MP4PARSE_OK
}

#[no_mangle]
pub unsafe extern "C" fn mp4parse_get_track_audio_info(context: *mut mp4parse_state, track: u32, info: *mut mp4parse_track_audio_info) -> mp4parse_error {
    if context.is_null() || info.is_null() {
        return MP4PARSE_ERROR_BADARG;
    }

    let context: &mut MediaContext = &mut (*context).0;

    if track as usize >= context.tracks.len() {
        return MP4PARSE_ERROR_BADARG;
    }

    let track = &context.tracks[track as usize];

    match track.track_type {
        TrackType::Audio => {}
        _ => return MP4PARSE_ERROR_INVALID,
    };

    let audio = match track.data {
        Some(ref data) => data,
        None => return MP4PARSE_ERROR_INVALID,
    };

    let audio = match *audio {
        SampleEntry::Audio(ref x) => x,
        _ => return MP4PARSE_ERROR_INVALID,
    };

    (*info).channels = audio.channelcount;
    (*info).bit_depth = audio.samplesize;
    (*info).sample_rate = audio.samplerate >> 16; // 16.16 fixed point

    MP4PARSE_OK
}

#[no_mangle]
pub unsafe extern "C" fn mp4parse_get_track_video_info(context: *mut mp4parse_state, track: u32, info: *mut mp4parse_track_video_info) -> mp4parse_error {
    if context.is_null() || info.is_null() {
        return MP4PARSE_ERROR_BADARG;
    }

    let context: &mut MediaContext = &mut (*context).0;

    if track as usize >= context.tracks.len() {
        return MP4PARSE_ERROR_BADARG;
    }

    let track = &context.tracks[track as usize];

    match track.track_type {
        TrackType::Video => {}
        _ => return MP4PARSE_ERROR_INVALID,
    };

    let video = match track.data {
        Some(ref data) => data,
        None => return MP4PARSE_ERROR_INVALID,
    };

    let video = match *video {
        SampleEntry::Video(ref x) => x,
        _ => return MP4PARSE_ERROR_INVALID,
    };

    if let Some(ref tkhd) = track.tkhd {
        (*info).display_width = tkhd.width >> 16; // 16.16 fixed point
        (*info).display_height = tkhd.height >> 16; // 16.16 fixed point
    } else {
        return MP4PARSE_ERROR_INVALID;
    }
    (*info).image_width = video.width;
    (*info).image_height = video.height;

    MP4PARSE_OK
}

#[test]
fn new_context() {
    let context = mp4parse_new();
    assert!(!context.is_null());
    unsafe {
        mp4parse_free(context);
    }
}

#[test]
#[should_panic(expected = "assertion failed")]
fn free_null_context() {
    unsafe {
        mp4parse_free(std::ptr::null_mut());
    }
}

#[test]
fn arg_validation() {
    let null_buffer = std::ptr::null();
    let null_context = std::ptr::null_mut();

    let context = mp4parse_new();
    assert!(!context.is_null());

    let buffer = vec![0u8; 8];

    unsafe {
        assert_eq!(MP4PARSE_ERROR_BADARG,
                   mp4parse_read(null_context, null_buffer, 0));
        assert_eq!(MP4PARSE_ERROR_BADARG,
                   mp4parse_read(context, null_buffer, 0));
    }

    for size in 0..buffer.len() {
        println!("testing buffer length {}", size);
        unsafe {
            assert_eq!(MP4PARSE_ERROR_BADARG,
                       mp4parse_read(context, buffer.as_ptr(), size));
        }
    }

    unsafe {
        mp4parse_free(context);
    }
}
