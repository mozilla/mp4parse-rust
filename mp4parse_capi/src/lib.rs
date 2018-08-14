//! C API for mp4parse module.
//!
//! Parses ISO Base Media Format aka video/mp4 streams.
//!
//! # Examples
//!
//! ```rust
//! extern crate mp4parse_capi;
//! use std::io::Read;
//!
//! extern fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
//!    let mut input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };
//!    let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
//!    match input.read(&mut buf) {
//!        Ok(n) => n as isize,
//!        Err(_) => -1,
//!    }
//! }
//!
//! let mut file = std::fs::File::open("../mp4parse/tests/minimal.mp4").unwrap();
//! let io = mp4parse_capi::Mp4parseIo {
//!     read: Some(buf_read),
//!     userdata: &mut file as *mut _ as *mut std::os::raw::c_void
//! };
//! unsafe {
//!     let parser = mp4parse_capi::mp4parse_new(&io);
//!     let rv = mp4parse_capi::mp4parse_read(parser);
//!     assert_eq!(rv, mp4parse_capi::Mp4parseStatus::Ok);
//!     mp4parse_capi::mp4parse_free(parser);
//! }
//! ```

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

extern crate mp4parse;
extern crate byteorder;
extern crate num_traits;

use std::io::Read;
use std::collections::HashMap;
use byteorder::WriteBytesExt;
use num_traits::{PrimInt, Zero};

// Symbols we need from our rust api.
use mp4parse::MediaContext;
use mp4parse::TrackType;
use mp4parse::read_mp4;
use mp4parse::Error;
use mp4parse::SampleEntry;
use mp4parse::AudioCodecSpecific;
use mp4parse::VideoCodecSpecific;
use mp4parse::MediaTimeScale;
use mp4parse::MediaScaledTime;
use mp4parse::TrackTimeScale;
use mp4parse::TrackScaledTime;
use mp4parse::serialize_opus_header;
use mp4parse::CodecType;
use mp4parse::Track;
use mp4parse::vec_push;

#[repr(C)]
#[derive(PartialEq, Debug)]
pub enum Mp4parseStatus {
    Ok = 0,
    BadArg = 1,
    Invalid = 2,
    Unsupported = 3,
    Eof = 4,
    Io = 5,
    Oom = 6,
}

#[repr(C)]
#[derive(PartialEq, Debug)]
pub enum Mp4parseTrackType {
    Video = 0,
    Audio = 1,
    Metadata = 2,
}

impl Default for Mp4parseTrackType {
    fn default() -> Self { Mp4parseTrackType::Video }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(PartialEq, Debug)]
pub enum Mp4parseCodec {
    Unknown,
    Aac,
    Flac,
    Opus,
    Avc,
    Vp9,
    Av1,
    Mp3,
    Mp4v,
    Jpeg,   // for QT JPEG atom in video track
    Ac3,
    Ec3,
    Alac,
}

impl Default for Mp4parseCodec {
    fn default() -> Self { Mp4parseCodec::Unknown }
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct Mp4parseTrackInfo {
    pub track_type: Mp4parseTrackType,
    pub codec: Mp4parseCodec,
    pub track_id: u32,
    pub duration: u64,
    pub media_time: i64, // wants to be u64? understand how elst adjustment works
    // TODO(kinetik): include crypto guff
}

#[repr(C)]
#[derive(Default, Debug, PartialEq)]
pub struct Mp4parseIndice {
    pub start_offset: u64,
    pub end_offset: u64,
    pub start_composition: i64,
    pub end_composition: i64,
    pub start_decode: i64,
    pub sync: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct Mp4parseByteData {
    pub length: u32,
    // cheddar can't handle generic type, so it needs to be multiple data types here.
    pub data: *const u8,
    pub indices: *const Mp4parseIndice,
}

impl Default for Mp4parseByteData {
    fn default() -> Self {
        Self {
            length: 0,
            data: std::ptr::null(),
            indices: std::ptr::null(),
        }
    }
}

impl Mp4parseByteData {
    fn set_data(&mut self, data: &[u8]) {
        self.length = data.len() as u32;
        self.data = data.as_ptr();
    }

    fn set_indices(&mut self, data: &[Mp4parseIndice]) {
        self.length = data.len() as u32;
        self.indices = data.as_ptr();
    }
}

#[repr(C)]
#[derive(Default)]
pub struct Mp4parsePsshInfo {
    pub data: Mp4parseByteData,
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct Mp4parseSinfInfo {
    pub is_encrypted: u32,
    pub iv_size: u8,
    pub kid: Mp4parseByteData,
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct Mp4parseTrackAudioInfo {
    pub channels: u16,
    pub bit_depth: u16,
    pub sample_rate: u32,
    pub profile: u16,
    pub codec_specific_config: Mp4parseByteData,
    pub extra_data: Mp4parseByteData,
    pub protected_data: Mp4parseSinfInfo,
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct Mp4parseTrackVideoInfo {
    pub display_width: u32,
    pub display_height: u32,
    pub image_width: u16,
    pub image_height: u16,
    pub rotation: u16,
    pub extra_data: Mp4parseByteData,
    pub protected_data: Mp4parseSinfInfo,
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct Mp4parseFragmentInfo {
    pub fragment_duration: u64,
    // TODO:
    // info in trex box.
}

pub struct Mp4parseParser {
    context: MediaContext,
    io: Mp4parseIo,
    poisoned: bool,
    opus_header: HashMap<u32, Vec<u8>>,
    pssh_data: Vec<u8>,
    sample_table: HashMap<u32, Vec<Mp4parseIndice>>,
}

impl Mp4parseParser {
    fn context(&self) -> &MediaContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut MediaContext {
        &mut self.context
    }

    fn io_mut(&mut self) -> &mut Mp4parseIo {
        &mut self.io
    }

    fn poisoned(&self) -> bool {
        self.poisoned
    }

    fn set_poisoned(&mut self, poisoned: bool) {
        self.poisoned = poisoned;
    }

    fn opus_header_mut(&mut self) -> &mut HashMap<u32, Vec<u8>> {
        &mut self.opus_header
    }

    fn pssh_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.pssh_data
    }

    fn sample_table_mut(&mut self) -> &mut HashMap<u32, Vec<Mp4parseIndice>> {
        &mut self.sample_table
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Mp4parseIo {
    pub read: Option<extern fn(buffer: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize>,
    pub userdata: *mut std::os::raw::c_void,
}

impl Read for Mp4parseIo {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() > isize::max_value() as usize {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "buf length overflow in Mp4parseIo Read impl"));
        }
        let rv = self.read.unwrap()(buf.as_mut_ptr(), buf.len(), self.userdata);
        if rv >= 0 {
            Ok(rv as usize)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "I/O error in Mp4parseIo Read impl"))
        }
    }
}

// C API wrapper functions.

/// Allocate an `Mp4parseParser*` to read from the supplied `Mp4parseIo`.
#[no_mangle]
pub unsafe extern fn mp4parse_new(io: *const Mp4parseIo) -> *mut Mp4parseParser {
    if io.is_null() || (*io).userdata.is_null() {
        return std::ptr::null_mut();
    }
    if (*io).read.is_none() {
        return std::ptr::null_mut();
    }
    let parser = Box::new(Mp4parseParser {
        context: MediaContext::new(),
        io: (*io).clone(),
        poisoned: false,
        opus_header: HashMap::new(),
        pssh_data: Vec::new(),
        sample_table: HashMap::new(),
    });

    Box::into_raw(parser)
}

/// Free an `Mp4parseParser*` allocated by `mp4parse_new()`.
#[no_mangle]
pub unsafe extern fn mp4parse_free(parser: *mut Mp4parseParser) {
    assert!(!parser.is_null());
    let _ = Box::from_raw(parser);
}

/// Run the `Mp4parseParser*` allocated by `mp4parse_new()` until EOF or error.
#[no_mangle]
pub unsafe extern fn mp4parse_read(parser: *mut Mp4parseParser) -> Mp4parseStatus {
    // Validate arguments from C.
    if parser.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    let context = (*parser).context_mut();
    let io = (*parser).io_mut();

    let r = read_mp4(io, context);
    match r {
        Ok(_) => Mp4parseStatus::Ok,
        Err(Error::NoMoov) | Err(Error::InvalidData(_)) => {
            // Block further calls. We've probable lost sync.
            (*parser).set_poisoned(true);
            Mp4parseStatus::Invalid
        }
        Err(Error::Unsupported(_)) => Mp4parseStatus::Unsupported,
        Err(Error::UnexpectedEOF) => Mp4parseStatus::Eof,
        Err(Error::Io(_)) => {
            // Block further calls after a read failure.
            // Getting std::io::ErrorKind::UnexpectedEof is normal
            // but our From trait implementation should have converted
            // those to our Error::UnexpectedEOF variant.
            (*parser).set_poisoned(true);
            Mp4parseStatus::Io
        },
        Err(Error::OutOfMemory) => Mp4parseStatus::Oom,
    }
}

/// Return the number of tracks parsed by previous `mp4parse_read()` call.
#[no_mangle]
pub unsafe extern fn mp4parse_get_track_count(parser: *const Mp4parseParser, count: *mut u32) -> Mp4parseStatus {
    // Validate arguments from C.
    if parser.is_null() || count.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }
    let context = (*parser).context();

    // Make sure the track count fits in a u32.
    if context.tracks.len() > u32::max_value() as usize {
        return Mp4parseStatus::Invalid;
    }
    *count = context.tracks.len() as u32;
    Mp4parseStatus::Ok
}

/// Calculate numerator * scale / denominator, if possible.
///
/// Applying the associativity of integer arithmetic, we divide first
/// and add the remainder after multiplying each term separately
/// to preserve precision while leaving more headroom. That is,
/// (n * s) / d is split into floor(n / d) * s + (n % d) * s / d.
///
/// Return None on overflow or if the denominator is zero.
fn rational_scale<T, S>(numerator: T, denominator: T, scale2: S) -> Option<T>
    where T: PrimInt + Zero, S: PrimInt {
    if denominator.is_zero() {
        return None;
    }

    let integer = numerator / denominator;
    let remainder = numerator % denominator;
    num_traits::cast(scale2).and_then(|s| {
        match integer.checked_mul(&s) {
            Some(integer) => remainder.checked_mul(&s)
                .and_then(|remainder| (remainder/denominator).checked_add(&integer)),
            None => None,
        }
    })
}

fn media_time_to_us(time: MediaScaledTime, scale: MediaTimeScale) -> Option<u64> {
    let microseconds_per_second = 1000000;
    rational_scale::<u64, u64>(time.0, scale.0, microseconds_per_second)
}

fn track_time_to_us<T>(time: TrackScaledTime<T>, scale: TrackTimeScale<T>) -> Option<T>
    where T: PrimInt + Zero {
    assert_eq!(time.1, scale.1);
    let microseconds_per_second = 1000000;
    rational_scale::<T, u64>(time.0, scale.0, microseconds_per_second)
}

/// Fill the supplied `Mp4parseTrackInfo` with metadata for `track`.
#[no_mangle]
pub unsafe extern fn mp4parse_get_track_info(parser: *mut Mp4parseParser, track_index: u32, info: *mut Mp4parseTrackInfo) -> Mp4parseStatus {
    if parser.is_null() || info.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    // Initialize fields to default values to ensure all fields are always valid.
    *info = Default::default();

    let context = (*parser).context_mut();
    let track_index: usize = track_index as usize;
    let info: &mut Mp4parseTrackInfo = &mut *info;

    if track_index >= context.tracks.len() {
        return Mp4parseStatus::BadArg;
    }

    info.track_type = match context.tracks[track_index].track_type {
        TrackType::Video => Mp4parseTrackType::Video,
        TrackType::Audio => Mp4parseTrackType::Audio,
        TrackType::Metadata => Mp4parseTrackType::Metadata,
        TrackType::Unknown => return Mp4parseStatus::Unsupported,
    };

    // Return UNKNOWN for unsupported format.
    info.codec = match context.tracks[track_index].data {
        Some(SampleEntry::Audio(ref audio)) => match audio.codec_specific {
            AudioCodecSpecific::OpusSpecificBox(_) =>
                Mp4parseCodec::Opus,
            AudioCodecSpecific::FLACSpecificBox(_) =>
                Mp4parseCodec::Flac,
            AudioCodecSpecific::ES_Descriptor(ref esds) if esds.audio_codec == CodecType::AAC =>
                Mp4parseCodec::Aac,
            AudioCodecSpecific::ES_Descriptor(ref esds) if esds.audio_codec == CodecType::MP3 =>
                Mp4parseCodec::Mp3,
            AudioCodecSpecific::ES_Descriptor(_) | AudioCodecSpecific::LPCM =>
                Mp4parseCodec::Unknown,
            AudioCodecSpecific::MP3 =>
                Mp4parseCodec::Mp3,
            AudioCodecSpecific::ALACSpecificBox(_) =>
                Mp4parseCodec::Alac,
        },
        Some(SampleEntry::Video(ref video)) => match video.codec_specific {
            VideoCodecSpecific::VPxConfig(_) =>
                Mp4parseCodec::Vp9,
            VideoCodecSpecific::AV1Config(_) =>
                Mp4parseCodec::Av1,
            VideoCodecSpecific::AVCConfig(_) =>
                Mp4parseCodec::Avc,
            VideoCodecSpecific::ESDSConfig(_) => // MP4V (14496-2) video is unsupported.
                Mp4parseCodec::Unknown,
        },
        _ => Mp4parseCodec::Unknown,
    };

    let track = &context.tracks[track_index];

    if let (Some(track_timescale),
            Some(context_timescale)) = (track.timescale,
                                        context.timescale) {
        let media_time =
            match track.media_time.map_or(Some(0), |media_time| {
                    track_time_to_us(media_time, track_timescale) }) {
                Some(time) => time as i64,
                None => return Mp4parseStatus::Invalid,
            };
        let empty_duration =
            match track.empty_duration.map_or(Some(0), |empty_duration| {
                    media_time_to_us(empty_duration, context_timescale) }) {
                Some(time) => time as i64,
                None => return Mp4parseStatus::Invalid,
            };
        info.media_time = media_time - empty_duration;

        if let Some(track_duration) = track.duration {
            match track_time_to_us(track_duration, track_timescale) {
                Some(duration) => info.duration = duration,
                None => return Mp4parseStatus::Invalid,
            }
        } else {
            // Duration unknown; stagefright returns 0 for this.
            info.duration = 0
        }
    } else {
        return Mp4parseStatus::Invalid
    }

    info.track_id = match track.track_id {
        Some(track_id) => track_id,
        None => return Mp4parseStatus::Invalid,
    };

    Mp4parseStatus::Ok
}

/// Fill the supplied `Mp4parseTrackAudioInfo` with metadata for `track`.
#[no_mangle]
pub unsafe extern fn mp4parse_get_track_audio_info(parser: *mut Mp4parseParser, track_index: u32, info: *mut Mp4parseTrackAudioInfo) -> Mp4parseStatus {
    if parser.is_null() || info.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    // Initialize fields to default values to ensure all fields are always valid.
    *info = Default::default();

    let context = (*parser).context_mut();

    if track_index as usize >= context.tracks.len() {
        return Mp4parseStatus::BadArg;
    }

    let track = &context.tracks[track_index as usize];

    match track.track_type {
        TrackType::Audio => {}
        _ => return Mp4parseStatus::Invalid,
    };

    let audio = match track.data {
        Some(ref data) => data,
        None => return Mp4parseStatus::Invalid,
    };

    let audio = match *audio {
        SampleEntry::Audio(ref x) => x,
        _ => return Mp4parseStatus::Invalid,
    };

    (*info).channels = audio.channelcount as u16;
    (*info).bit_depth = audio.samplesize;
    (*info).sample_rate = audio.samplerate as u32;

    match audio.codec_specific {
        AudioCodecSpecific::ES_Descriptor(ref v) => {
            if v.codec_esds.len() > std::u32::MAX as usize {
                return Mp4parseStatus::Invalid;
            }
            (*info).extra_data.length = v.codec_esds.len() as u32;
            (*info).extra_data.data = v.codec_esds.as_ptr();
            (*info).codec_specific_config.length = v.decoder_specific_data.len() as u32;
            (*info).codec_specific_config.data = v.decoder_specific_data.as_ptr();
            if let Some(rate) = v.audio_sample_rate {
                (*info).sample_rate = rate;
            }
            if let Some(channels) = v.audio_channel_count {
                (*info).channels = channels;
            }
            if let Some(profile) = v.audio_object_type {
                (*info).profile = profile;
            }
        }
        AudioCodecSpecific::FLACSpecificBox(ref flac) => {
            // Return the STREAMINFO metadata block in the codec_specific.
            let streaminfo = &flac.blocks[0];
            if streaminfo.block_type != 0 || streaminfo.data.len() != 34 {
                return Mp4parseStatus::Invalid;
            }
            (*info).codec_specific_config.length = streaminfo.data.len() as u32;
            (*info).codec_specific_config.data = streaminfo.data.as_ptr();
        }
        AudioCodecSpecific::OpusSpecificBox(ref opus) => {
            let mut v = Vec::new();
            match serialize_opus_header(opus, &mut v) {
                Err(_) => {
                    return Mp4parseStatus::Invalid;
                }
                Ok(_) => {
                    let header = (*parser).opus_header_mut();
                    header.insert(track_index, v);
                    if let Some(v) = header.get(&track_index) {
                        if v.len() > std::u32::MAX as usize {
                            return Mp4parseStatus::Invalid;
                        }
                        (*info).codec_specific_config.length = v.len() as u32;
                        (*info).codec_specific_config.data = v.as_ptr();
                    }
                }
            }
        }
        AudioCodecSpecific::ALACSpecificBox(ref alac) => {
            (*info).codec_specific_config.length = alac.data.len() as u32;
            (*info).codec_specific_config.data = alac.data.as_ptr();
        }
        AudioCodecSpecific::MP3 | AudioCodecSpecific::LPCM => (),
    }

    if let Some(p) = audio.protection_info.iter().find(|sinf| sinf.tenc.is_some()) {
        if let Some(ref tenc) = p.tenc {
            (*info).protected_data.is_encrypted = tenc.is_encrypted;
            (*info).protected_data.iv_size = tenc.iv_size;
            (*info).protected_data.kid.set_data(&(tenc.kid));
        }
    }

    Mp4parseStatus::Ok
}

/// Fill the supplied `Mp4parseTrackVideoInfo` with metadata for `track`.
#[no_mangle]
pub unsafe extern fn mp4parse_get_track_video_info(parser: *mut Mp4parseParser, track_index: u32, info: *mut Mp4parseTrackVideoInfo) -> Mp4parseStatus {
    if parser.is_null() || info.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    // Initialize fields to default values to ensure all fields are always valid.
    *info = Default::default();

    let context = (*parser).context_mut();

    if track_index as usize >= context.tracks.len() {
        return Mp4parseStatus::BadArg;
    }

    let track = &context.tracks[track_index as usize];

    match track.track_type {
        TrackType::Video => {}
        _ => return Mp4parseStatus::Invalid,
    };

    let video = match track.data {
        Some(ref data) => data,
        None => return Mp4parseStatus::Invalid,
    };

    let video = match *video {
        SampleEntry::Video(ref x) => x,
        _ => return Mp4parseStatus::Invalid,
    };

    if let Some(ref tkhd) = track.tkhd {
        (*info).display_width = tkhd.width >> 16; // 16.16 fixed point
        (*info).display_height = tkhd.height >> 16; // 16.16 fixed point
        let matrix = (tkhd.matrix.a >> 16, tkhd.matrix.b >> 16,
                      tkhd.matrix.c >> 16, tkhd.matrix.d >> 16);
        (*info).rotation = match matrix {
            ( 0,  1, -1,  0) => 90, // rotate 90 degrees
            (-1,  0,  0, -1) => 180, // rotate 180 degrees
            ( 0, -1,  1,  0) => 270, // rotate 270 degrees
            _ => 0,
        };
    } else {
        return Mp4parseStatus::Invalid;
    }
    (*info).image_width = video.width;
    (*info).image_height = video.height;

    match video.codec_specific {
        VideoCodecSpecific::AVCConfig(ref data) | VideoCodecSpecific::ESDSConfig(ref data) => {
          (*info).extra_data.set_data(data);
        },
        _ => {}
    }

    if let Some(p) = video.protection_info.iter().find(|sinf| sinf.tenc.is_some()) {
        if let Some(ref tenc) = p.tenc {
            (*info).protected_data.is_encrypted = tenc.is_encrypted;
            (*info).protected_data.iv_size = tenc.iv_size;
            (*info).protected_data.kid.set_data(&(tenc.kid));
        }
    }

    Mp4parseStatus::Ok
}

#[no_mangle]
pub unsafe extern fn mp4parse_get_indice_table(parser: *mut Mp4parseParser, track_id: u32, indices: *mut Mp4parseByteData) -> Mp4parseStatus {
    if parser.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    // Initialize fields to default values to ensure all fields are always valid.
    *indices = Default::default();

    let context = (*parser).context();
    let tracks = &context.tracks;
    let track = match tracks.iter().find(|track| track.track_id == Some(track_id)) {
        Some(t) => t,
        _ => return Mp4parseStatus::Invalid,
    };

    let index_table = (*parser).sample_table_mut();
    if let Some(v) = index_table.get(&track_id) {
        (*indices).set_indices(v);
        return Mp4parseStatus::Ok;
    }

    let media_time = match (&track.media_time, &track.timescale) {
        (&Some(t), &Some(s)) => {
            track_time_to_us(t, s).map(|v| v as i64)
        },
        _ => None,
    };

    let empty_duration = match (&track.empty_duration, &context.timescale) {
        (&Some(e), &Some(s)) => {
            media_time_to_us(e, s).map(|v| v as i64)
        },
        _ => None
    };

    // Find the track start offset time from 'elst'.
    // 'media_time' maps start time onward, 'empty_duration' adds time offset
    // before first frame is displayed.
    let offset_time = match (empty_duration, media_time) {
        (Some(e), Some(m)) => e - m,
        (Some(e), None) => e,
        (None, Some(m)) => m,
        _ => 0,
    };

    if let Some(v) = create_sample_table(track, offset_time) {
        (*indices).set_indices(&v);
        index_table.insert(track_id, v);
        return Mp4parseStatus::Ok;
    }

    Mp4parseStatus::Invalid
}

// Convert a 'ctts' compact table to full table by iterator,
// (sample_with_the_same_offset_count, offset) => (offset), (offset), (offset) ...
//
// For example:
// (2, 10), (4, 9) into (10, 10, 9, 9, 9, 9) by calling next_offset_time().
struct TimeOffsetIterator<'a> {
    cur_sample_range: std::ops::Range<u32>,
    cur_offset: i64,
    ctts_iter: Option<std::slice::Iter<'a, mp4parse::TimeOffset>>,
    track_id: usize,
}

impl<'a> Iterator for TimeOffsetIterator<'a> {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        let has_sample = self.cur_sample_range.next()
            .or_else(|| {
                // At end of current TimeOffset, find the next TimeOffset.
                let iter = match self.ctts_iter {
                    Some(ref mut v) => v,
                    _ => return None,
                };
                let offset_version;
                self.cur_sample_range = match iter.next() {
                    Some(v) => {
                        offset_version = v.time_offset;
                        (0 .. v.sample_count)
                    },
                    _ => {
                        offset_version = mp4parse::TimeOffsetVersion::Version0(0);
                        (0 .. 0)
                    },
                };

                self.cur_offset = match offset_version {
                    mp4parse::TimeOffsetVersion::Version0(i) => i as i64,
                    mp4parse::TimeOffsetVersion::Version1(i) => i as i64,
                };

                self.cur_sample_range.next()
            });

        has_sample.and(Some(self.cur_offset))
    }
}

impl<'a> TimeOffsetIterator<'a> {
    fn next_offset_time(&mut self) -> TrackScaledTime<i64> {
        match self.next() {
            Some(v) => TrackScaledTime::<i64>(v as i64, self.track_id),
            _ => TrackScaledTime::<i64>(0, self.track_id),
        }
    }
}

// Convert 'stts' compact table to full table by iterator,
// (sample_count_with_the_same_time, time) => (time, time, time) ... repeats
// sample_count_with_the_same_time.
//
// For example:
// (2, 3000), (1, 2999) to (3000, 3000, 2999).
struct TimeToSampleIterator<'a> {
    cur_sample_count: std::ops::Range<u32>,
    cur_sample_delta: u32,
    stts_iter: std::slice::Iter<'a, mp4parse::Sample>,
    track_id: usize,
}

impl<'a> Iterator for TimeToSampleIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let has_sample = self.cur_sample_count.next()
            .or_else(|| {
                self.cur_sample_count = match self.stts_iter.next() {
                    Some(v) => {
                        self.cur_sample_delta = v.sample_delta;
                        (0 .. v.sample_count)
                    },
                    _ => (0 .. 0),
                };

                self.cur_sample_count.next()
            });

        has_sample.and(Some(self.cur_sample_delta))
    }
}

impl<'a> TimeToSampleIterator<'a> {
    fn next_delta(&mut self) -> TrackScaledTime<i64> {
        match self.next() {
            Some(v) => TrackScaledTime::<i64>(v as i64, self.track_id),
            _ => TrackScaledTime::<i64>(0, self.track_id),
        }
    }
}

// Convert 'stco' compact table to full table by iterator.
// (start_chunk_num, sample_number) => (start_chunk_num, sample_number),
//                                     (start_chunk_num + 1, sample_number),
//                                     (start_chunk_num + 2, sample_number),
//                                     ...
//                                     (next start_chunk_num, next sample_number),
//                                     ...
//
// For example:
// (1, 5), (5, 10), (9, 2) => (1, 5), (2, 5), (3, 5), (4, 5), (5, 10), (6, 10),
// (7, 10), (8, 10), (9, 2)
struct SampleToChunkIterator<'a> {
    chunks: std::ops::Range<u32>,
    sample_count: u32,
    stsc_peek_iter: std::iter::Peekable<std::slice::Iter<'a, mp4parse::SampleToChunk>>,
    remain_chunk_count: u32, // total chunk number from 'stco'.
}

impl<'a> Iterator for SampleToChunkIterator<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        let has_chunk = self.chunks.next()
            .or_else(|| {
                self.chunks = self.locate();
                self.remain_chunk_count.checked_sub(self.chunks.len() as u32).and_then(|res| {
                    self.remain_chunk_count = res;
                    self.chunks.next()
                })
            });

        has_chunk.map_or(None, |id| { Some((id, self.sample_count)) })
    }
}

impl<'a> SampleToChunkIterator<'a> {
    fn locate(&mut self) -> std::ops::Range<u32> {
        loop {
            return match (self.stsc_peek_iter.next(), self.stsc_peek_iter.peek()) {
                (Some(next), Some(peek)) if next.first_chunk == peek.first_chunk => {
                    // Invalid entry, skip it and will continue searching at
                    // next loop iteration.
                    continue
                },
                (Some(next), Some(peek)) if next.first_chunk > 0 && peek.first_chunk > 0 => {
                    self.sample_count = next.samples_per_chunk;
                    (next.first_chunk - 1) .. (peek.first_chunk - 1)
                },
                (Some(next), None) if next.first_chunk > 0 => {
                    self.sample_count = next.samples_per_chunk;
                    // Total chunk number in 'stsc' could be different to 'stco',
                    // there could be more chunks at the last 'stsc' record.
                    match next.first_chunk.checked_add(self.remain_chunk_count) {
                        Some(r) => (next.first_chunk - 1) .. r - 1,
                        _ => 0 .. 0,
                    }
                },
                _ => 0 .. 0
            };
        };
    }
}

fn create_sample_table(track: &Track, track_offset_time: i64) -> Option<Vec<Mp4parseIndice>> {
    let timescale = match track.timescale {
        Some(ref t) => TrackTimeScale::<i64>(t.0 as i64, t.1),
        _ => TrackTimeScale::<i64>(0, 0),
    };

    let (stsc, stco, stsz, stts) =
        match (&track.stsc, &track.stco, &track.stsz, &track.stts) {
            (&Some(ref a), &Some(ref b), &Some(ref c), &Some(ref d)) => (a, b, c, d),
            _ => return None,
        };

    // According to spec, no sync table means every sample is sync sample.
    let has_sync_table = match track.stss {
        Some(_) => true,
        _ => false,
    };

    let mut sample_table = Vec::new();
    let mut sample_size_iter = stsz.sample_sizes.iter();

    // Get 'stsc' iterator for (chunk_id, chunk_sample_count) and calculate the sample
    // offset address.
    let stsc_iter = SampleToChunkIterator {
        chunks: (0 .. 0),
        sample_count: 0,
        stsc_peek_iter: stsc.samples.as_slice().iter().peekable(),
        remain_chunk_count: stco.offsets.len() as u32,
    };

    for i in stsc_iter {
        let chunk_id = i.0 as usize;
        let sample_counts = i.1;
        let mut cur_position = match stco.offsets.get(chunk_id) {
            Some(&i) => i,
            _ => return None,
        };
        for _ in 0 .. sample_counts {
            let start_offset = cur_position;
            let end_offset = match (stsz.sample_size, sample_size_iter.next()) {
                (_, Some(t)) => start_offset + *t as u64,
                (t, _) if t > 0 => start_offset + t as u64,
                _ => 0,
            };
            if end_offset == 0 {
                return None;
            }
            cur_position = end_offset;

            let res = vec_push(&mut sample_table, Mp4parseIndice {
                start_offset: start_offset,
                end_offset: end_offset,
                start_composition: 0,
                end_composition: 0,
                start_decode: 0,
                sync: !has_sync_table,
            });
            if res.is_err() {
                return None;
            }
        }
    }

    // Mark the sync sample in sample_table according to 'stss'.
    if let Some(ref v) = track.stss {
        for iter in &v.samples {
            match iter.checked_sub(1).and_then(|idx| { sample_table.get_mut(idx as usize) }) {
                Some(elem) => elem.sync = true,
                _ => return None,
            }
        }
    }

    let ctts_iter = match track.ctts {
        Some(ref v) => Some(v.samples.as_slice().iter()),
        _ => None,
    };

    let mut ctts_offset_iter = TimeOffsetIterator {
        cur_sample_range: (0 .. 0),
        cur_offset: 0,
        ctts_iter: ctts_iter,
        track_id: track.id,
    };

    let mut stts_iter = TimeToSampleIterator {
        cur_sample_count: (0 .. 0),
        cur_sample_delta: 0,
        stts_iter: stts.samples.as_slice().iter(),
        track_id: track.id,
    };

    // sum_delta is the sum of stts_iter delta.
    // According to sepc:
    //      decode time => DT(n) = DT(n-1) + STTS(n)
    //      composition time => CT(n) = DT(n) + CTTS(n)
    // Note:
    //      composition time needs to add the track offset time from 'elst' table.
    let mut sum_delta = TrackScaledTime::<i64>(0, track.id);
    for sample in sample_table.as_mut_slice() {
        let decode_time = sum_delta;
        sum_delta = sum_delta + stts_iter.next_delta();

        // ctts_offset is the current sample offset time.
        let ctts_offset = ctts_offset_iter.next_offset_time();

        let start_composition = track_time_to_us(decode_time + ctts_offset, timescale);

        let end_composition = track_time_to_us(sum_delta + ctts_offset, timescale);

        let start_decode = track_time_to_us(decode_time, timescale);

        match (start_composition, end_composition, start_decode) {
            (Some(s_c), Some(e_c), Some(s_d)) => {
                sample.start_composition = s_c + track_offset_time;
                sample.end_composition = e_c + track_offset_time;
                sample.start_decode = s_d;
            },
            _ => return None,
        }
    }

    // Correct composition end time due to 'ctts' causes composition time re-ordering.
    //
    // Composition end time is not in specification. However, gecko needs it, so we need to
    // calculate to correct the composition end time.
    if sample_table.len() > 0 {
        // Create an index table refers to sample_table and sorted by start_composisiton time.
        let mut sort_table = Vec::new();
        for i in 0 .. sample_table.len() {
            if vec_push(&mut sort_table, i).is_err() {
                return None;
            }
        }

        sort_table.sort_by_key(|i| {
            match sample_table.get(*i) {
                Some(v) => {
                    v.start_composition
                },
                _ => 0,
            }
        });

        let iter = sort_table.iter();
        for i in 0 .. (iter.len() - 1) {
            let current_index = sort_table[i];
            let peek_index = sort_table[i + 1];
            let next_start_composition_time = sample_table[peek_index].start_composition;
            let sample = &mut sample_table[current_index];
            sample.end_composition = next_start_composition_time;
        }
    }

    Some(sample_table)
}

/// Fill the supplied `Mp4parseFragmentInfo` with metadata from fragmented file.
#[no_mangle]
pub unsafe extern fn mp4parse_get_fragment_info(parser: *mut Mp4parseParser, info: *mut Mp4parseFragmentInfo) -> Mp4parseStatus {
    if parser.is_null() || info.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    // Initialize fields to default values to ensure all fields are always valid.
    *info = Default::default();

    let context = (*parser).context();
    let info: &mut Mp4parseFragmentInfo = &mut *info;

    info.fragment_duration = 0;

    let duration = match context.mvex {
        Some(ref mvex) => mvex.fragment_duration,
        None => return Mp4parseStatus::Invalid,
    };

    if let (Some(time), Some(scale)) = (duration, context.timescale) {
        info.fragment_duration = match media_time_to_us(time, scale) {
            Some(time_us) => time_us as u64,
            None => return Mp4parseStatus::Invalid,
        }
    }

    Mp4parseStatus::Ok
}

/// A fragmented file needs mvex table and contains no data in stts, stsc, and stco boxes.
#[no_mangle]
pub unsafe extern fn mp4parse_is_fragmented(parser: *mut Mp4parseParser, track_id: u32, fragmented: *mut u8) -> Mp4parseStatus {
    if parser.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    let context = (*parser).context_mut();
    let tracks = &context.tracks;
    (*fragmented) = false as u8;

    if context.mvex.is_none() {
        return Mp4parseStatus::Ok;
    }

    // check sample tables.
    let mut iter = tracks.iter();
    iter.find(|track| track.track_id == Some(track_id)).map_or(Mp4parseStatus::BadArg, |track| {
        match (&track.stsc, &track.stco, &track.stts) {
            (&Some(ref stsc), &Some(ref stco), &Some(ref stts))
                if stsc.samples.is_empty() && stco.offsets.is_empty() && stts.samples.is_empty() => (*fragmented) = true as u8,
            _ => {},
        };
        Mp4parseStatus::Ok
    })
}

/// Get 'pssh' system id and 'pssh' box content for eme playback.
///
/// The data format of the `info` struct passed to gecko is:
///
/// - system id (16 byte uuid)
/// - pssh box size (32-bit native endian)
/// - pssh box content (including header)
#[no_mangle]
pub unsafe extern fn mp4parse_get_pssh_info(parser: *mut Mp4parseParser, info: *mut Mp4parsePsshInfo) -> Mp4parseStatus {
    if parser.is_null() || info.is_null() || (*parser).poisoned() {
        return Mp4parseStatus::BadArg;
    }

    // Initialize fields to default values to ensure all fields are always valid.
    *info = Default::default();

    let context = (*parser).context_mut();
    let pssh_data = (*parser).pssh_data_mut();
    let info: &mut Mp4parsePsshInfo = &mut *info;

    pssh_data.clear();
    for pssh in &context.psshs {
        let content_len = pssh.box_content.len();
        if content_len > std::u32::MAX as usize {
            return Mp4parseStatus::Invalid;
        }
        let mut data_len = Vec::new();
        if data_len.write_u32::<byteorder::NativeEndian>(content_len as u32).is_err() {
            return Mp4parseStatus::Io;
        }
        pssh_data.extend_from_slice(pssh.system_id.as_slice());
        pssh_data.extend_from_slice(data_len.as_slice());
        pssh_data.extend_from_slice(pssh.box_content.as_slice());
    }

    info.data.set_data(pssh_data);

    Mp4parseStatus::Ok
}

#[cfg(test)]
extern fn panic_read(_: *mut u8, _: usize, _: *mut std::os::raw::c_void) -> isize {
    panic!("panic_read shouldn't be called in these tests");
}

#[cfg(test)]
extern fn error_read(_: *mut u8, _: usize, _: *mut std::os::raw::c_void) -> isize {
    -1
}

#[cfg(test)]
extern fn valid_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };

    let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(&mut buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[test]
fn new_parser() {
    let mut dummy_value: u32 = 42;
    let io = Mp4parseIo {
        read: Some(panic_read),
        userdata: &mut dummy_value as *mut _ as *mut std::os::raw::c_void,
    };
    unsafe {
        let parser = mp4parse_new(&io);
        assert!(!parser.is_null());
        mp4parse_free(parser);
    }
}

#[test]
fn get_track_count_null_parser() {
    unsafe {
        let mut count: u32 = 0;
        let rv = mp4parse_get_track_count(std::ptr::null(), std::ptr::null_mut());
        assert_eq!(rv, Mp4parseStatus::BadArg);
        let rv = mp4parse_get_track_count(std::ptr::null(), &mut count);
        assert_eq!(rv, Mp4parseStatus::BadArg);
    }
}

#[test]
fn arg_validation() {
    unsafe {
        // Passing a null Mp4parseIo is an error.
        let parser = mp4parse_new(std::ptr::null());
        assert!(parser.is_null());

        let null_mut: *mut std::os::raw::c_void = std::ptr::null_mut();

        // Passing an Mp4parseIo with null members is an error.
        let io = Mp4parseIo { read: None,
                               userdata: null_mut };
        let parser = mp4parse_new(&io);
        assert!(parser.is_null());

        let io = Mp4parseIo { read: Some(panic_read),
                               userdata: null_mut };
        let parser = mp4parse_new(&io);
        assert!(parser.is_null());

        let mut dummy_value = 42;
        let io = Mp4parseIo {
            read: None,
            userdata: &mut dummy_value as *mut _ as *mut std::os::raw::c_void,
        };
        let parser = mp4parse_new(&io);
        assert!(parser.is_null());

        // Passing a null Mp4parseParser is an error.
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_read(std::ptr::null_mut()));

        let mut dummy_info = Mp4parseTrackInfo {
            track_type: Mp4parseTrackType::Video,
            codec: Mp4parseCodec::Unknown,
            track_id: 0,
            duration: 0,
            media_time: 0,
        };
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_info(std::ptr::null_mut(), 0, &mut dummy_info));

        let mut dummy_video = Mp4parseTrackVideoInfo {
            display_width: 0,
            display_height: 0,
            image_width: 0,
            image_height: 0,
            rotation: 0,
            extra_data: Mp4parseByteData::default(),
            protected_data: Default::default(),
        };
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_video_info(std::ptr::null_mut(), 0, &mut dummy_video));

        let mut dummy_audio = Default::default();
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_audio_info(std::ptr::null_mut(), 0, &mut dummy_audio));
    }
}

#[test]
fn arg_validation_with_parser() {
    unsafe {
        let mut dummy_value = 42;
        let io = Mp4parseIo {
            read: Some(error_read),
            userdata: &mut dummy_value as *mut _ as *mut std::os::raw::c_void,
        };
        let parser = mp4parse_new(&io);
        assert!(!parser.is_null());

        // Our Mp4parseIo read should simply fail with an error.
        assert_eq!(Mp4parseStatus::Io, mp4parse_read(parser));

        // The parser is now poisoned and unusable.
        assert_eq!(Mp4parseStatus::BadArg,  mp4parse_read(parser));

        // Null info pointers are an error.
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_info(parser, 0, std::ptr::null_mut()));
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_video_info(parser, 0, std::ptr::null_mut()));
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_audio_info(parser, 0, std::ptr::null_mut()));

        let mut dummy_info = Mp4parseTrackInfo {
            track_type: Mp4parseTrackType::Video,
            codec: Mp4parseCodec::Unknown,
            track_id: 0,
            duration: 0,
            media_time: 0,
        };
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_info(parser, 0, &mut dummy_info));

        let mut dummy_video = Mp4parseTrackVideoInfo {
            display_width: 0,
            display_height: 0,
            image_width: 0,
            image_height: 0,
            rotation: 0,
            extra_data: Mp4parseByteData::default(),
            protected_data: Default::default(),
        };
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_video_info(parser, 0, &mut dummy_video));

        let mut dummy_audio = Default::default();
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_audio_info(parser, 0, &mut dummy_audio));

        mp4parse_free(parser);
    }
}

#[test]
fn get_track_count_poisoned_parser() {
    unsafe {
        let mut dummy_value = 42;
        let io = Mp4parseIo {
            read: Some(error_read),
            userdata: &mut dummy_value as *mut _ as *mut std::os::raw::c_void,
        };
        let parser = mp4parse_new(&io);
        assert!(!parser.is_null());

        // Our Mp4parseIo read should simply fail with an error.
        assert_eq!(Mp4parseStatus::Io, mp4parse_read(parser));

        let mut count: u32 = 0;
        let rv = mp4parse_get_track_count(parser, &mut count);
        assert_eq!(rv, Mp4parseStatus::BadArg);

        mp4parse_free(parser);
    }
}

#[test]
fn arg_validation_with_data() {
    unsafe {
        let mut file = std::fs::File::open("../mp4parse/tests/minimal.mp4").unwrap();
        let io = Mp4parseIo { read: Some(valid_read),
                               userdata: &mut file as *mut _ as *mut std::os::raw::c_void };
        let parser = mp4parse_new(&io);
        assert!(!parser.is_null());

        assert_eq!(Mp4parseStatus::Ok, mp4parse_read(parser));

        let mut count: u32 = 0;
        assert_eq!(Mp4parseStatus::Ok, mp4parse_get_track_count(parser, &mut count));
        assert_eq!(2, count);

        let mut info = Mp4parseTrackInfo {
            track_type: Mp4parseTrackType::Video,
            codec: Mp4parseCodec::Unknown,
            track_id: 0,
            duration: 0,
            media_time: 0,
        };
        assert_eq!(Mp4parseStatus::Ok, mp4parse_get_track_info(parser, 0, &mut info));
        assert_eq!(info.track_type, Mp4parseTrackType::Video);
        assert_eq!(info.codec, Mp4parseCodec::Avc);
        assert_eq!(info.track_id, 1);
        assert_eq!(info.duration, 40000);
        assert_eq!(info.media_time, 0);

        assert_eq!(Mp4parseStatus::Ok, mp4parse_get_track_info(parser, 1, &mut info));
        assert_eq!(info.track_type, Mp4parseTrackType::Audio);
        assert_eq!(info.codec, Mp4parseCodec::Aac);
        assert_eq!(info.track_id, 2);
        assert_eq!(info.duration, 61333);
        assert_eq!(info.media_time, 21333);

        let mut video = Mp4parseTrackVideoInfo {
            display_width: 0,
            display_height: 0,
            image_width: 0,
            image_height: 0,
            rotation: 0,
            extra_data: Mp4parseByteData::default(),
            protected_data: Default::default(),
        };
        assert_eq!(Mp4parseStatus::Ok, mp4parse_get_track_video_info(parser, 0, &mut video));
        assert_eq!(video.display_width, 320);
        assert_eq!(video.display_height, 240);
        assert_eq!(video.image_width, 320);
        assert_eq!(video.image_height, 240);

        let mut audio = Default::default();
        assert_eq!(Mp4parseStatus::Ok, mp4parse_get_track_audio_info(parser, 1, &mut audio));
        assert_eq!(audio.channels, 1);
        assert_eq!(audio.bit_depth, 16);
        assert_eq!(audio.sample_rate, 48000);

        // Test with an invalid track number.
        let mut info = Mp4parseTrackInfo {
            track_type: Mp4parseTrackType::Video,
            codec: Mp4parseCodec::Unknown,
            track_id: 0,
            duration: 0,
            media_time: 0,
        };
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_info(parser, 3, &mut info));
        assert_eq!(info.track_type, Mp4parseTrackType::Video);
        assert_eq!(info.codec, Mp4parseCodec::Unknown);
        assert_eq!(info.track_id, 0);
        assert_eq!(info.duration, 0);
        assert_eq!(info.media_time, 0);

        let mut video = Mp4parseTrackVideoInfo {
            display_width: 0,
            display_height: 0,
            image_width: 0,
            image_height: 0,
            rotation: 0,
            extra_data: Mp4parseByteData::default(),
            protected_data: Default::default(),
        };
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_video_info(parser, 3, &mut video));
        assert_eq!(video.display_width, 0);
        assert_eq!(video.display_height, 0);
        assert_eq!(video.image_width, 0);
        assert_eq!(video.image_height, 0);

        let mut audio = Default::default();
        assert_eq!(Mp4parseStatus::BadArg, mp4parse_get_track_audio_info(parser, 3, &mut audio));
        assert_eq!(audio.channels, 0);
        assert_eq!(audio.bit_depth, 0);
        assert_eq!(audio.sample_rate, 0);

        mp4parse_free(parser);
    }
}

#[test]
fn rational_scale_overflow() {
    assert_eq!(rational_scale::<u64, u64>(17, 3, 1000), Some(5666));
    let large = 0x4000_0000_0000_0000;
    assert_eq!(rational_scale::<u64, u64>(large, 2, 2), Some(large));
    assert_eq!(rational_scale::<u64, u64>(large, 4, 4), Some(large));
    assert_eq!(rational_scale::<u64, u64>(large, 2, 8), None);
    assert_eq!(rational_scale::<u64, u64>(large, 8, 4), Some(large/2));
    assert_eq!(rational_scale::<u64, u64>(large + 1, 4, 4), Some(large+1));
    assert_eq!(rational_scale::<u64, u64>(large, 40, 1000), None);
}

#[test]
fn media_time_overflow() {
  let scale = MediaTimeScale(90000);
  let duration = MediaScaledTime(9007199254710000);
  assert_eq!(media_time_to_us(duration, scale), Some(100079991719000000));
}

#[test]
fn track_time_overflow() {
  let scale = TrackTimeScale(44100u64, 0);
  let duration = TrackScaledTime(4413527634807900u64, 0);
  assert_eq!(track_time_to_us(duration, scale), Some(100079991719000000));
}
