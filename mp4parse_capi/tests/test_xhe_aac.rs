// Test for xHE-AAC codec detection via C API

use mp4parse_capi::*;
use std::fs::File;
use std::io::Read;

static XHE_AAC_FILE: &str = "tests/sine-3s-xhe-aac-44khz-mono.mp4";

extern "C" fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut File = unsafe { &mut *(userdata as *mut _) };
    let buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[test]
fn test_xhe_aac_codec_detection() {
    unsafe {
        let mut file = File::open(XHE_AAC_FILE).expect("Failed to open xHE-AAC test file");
        let io = Mp4parseIo {
            read: Some(buf_read),
            userdata: &mut file as *mut _ as *mut std::os::raw::c_void,
        };

        let mut parser = std::ptr::null_mut();
        let rv = mp4parse_new(&io, &mut parser);
        assert_eq!(rv, Mp4parseStatus::Ok);
        assert!(!parser.is_null());

        let mut count: u32 = 0;
        let rv = mp4parse_get_track_count(parser, &mut count);
        assert_eq!(rv, Mp4parseStatus::Ok);
        assert_eq!(count, 1, "Expected exactly one track");

        let mut track_info = Mp4parseTrackInfo::default();
        let rv = mp4parse_get_track_info(parser, 0, &mut track_info);
        assert_eq!(rv, Mp4parseStatus::Ok);
        assert_eq!(track_info.track_type, Mp4parseTrackType::Audio);

        let mut audio_info = Mp4parseTrackAudioInfo::default();
        let rv = mp4parse_get_track_audio_info(parser, 0, &mut audio_info);
        assert_eq!(rv, Mp4parseStatus::Ok);
        assert_eq!(audio_info.sample_info_count, 1);

        let sample_info = &*audio_info.sample_info;
        assert_eq!(sample_info.codec_type, Mp4parseCodec::XHEAAC);

        assert_eq!(sample_info.channels, 1); // mono
        assert_eq!(sample_info.sample_rate, 44100); // 44.1kHz
        assert_eq!(sample_info.profile, 42); // audio object type 42 (xHE-AAC)
        assert_eq!(sample_info.extended_profile, 42);

        assert!(sample_info.codec_specific_config.length > 0);
        assert!(!sample_info.codec_specific_config.data.is_null());

        mp4parse_free(parser);
    }
}

#[test]
fn test_xhe_aac_vs_regular_aac() {
    unsafe {
        let mut file = File::open(XHE_AAC_FILE).expect("Failed to open xHE-AAC test file");
        let io = Mp4parseIo {
            read: Some(buf_read),
            userdata: &mut file as *mut _ as *mut std::os::raw::c_void,
        };

        let mut parser = std::ptr::null_mut();
        let rv = mp4parse_new(&io, &mut parser);
        assert_eq!(rv, Mp4parseStatus::Ok);

        let mut audio_info = Mp4parseTrackAudioInfo::default();
        let rv = mp4parse_get_track_audio_info(parser, 0, &mut audio_info);
        assert_eq!(rv, Mp4parseStatus::Ok);

        let sample_info = &*audio_info.sample_info;
        assert_eq!(sample_info.codec_type, Mp4parseCodec::XHEAAC);
        assert_eq!(sample_info.profile, 42); // xHE-AAC audio object type

        mp4parse_free(parser);
    }
}

#[test]
fn test_xhe_aac_codec_not_unknown() {
    unsafe {
        let mut file = File::open(XHE_AAC_FILE).expect("Failed to open xHE-AAC test file");
        let io = Mp4parseIo {
            read: Some(buf_read),
            userdata: &mut file as *mut _ as *mut std::os::raw::c_void,
        };

        let mut parser = std::ptr::null_mut();
        let rv = mp4parse_new(&io, &mut parser);
        assert_eq!(rv, Mp4parseStatus::Ok);

        let mut audio_info = Mp4parseTrackAudioInfo::default();
        let rv = mp4parse_get_track_audio_info(parser, 0, &mut audio_info);
        assert_eq!(rv, Mp4parseStatus::Ok);

        let sample_info = &*audio_info.sample_info;

        assert_ne!(
            sample_info.codec_type,
            Mp4parseCodec::Unknown,
            "xHE-AAC should not be detected as Unknown codec"
        );

        assert_eq!(
            sample_info.codec_type,
            Mp4parseCodec::XHEAAC,
            "xHE-AAC should be detected as XHEAAC codec"
        );

        mp4parse_free(parser);
    }
}
