// Test for afconvert AAC duration issue #404
// https://github.com/mozilla/mp4parse-rust/issues/404

use mp4parse_capi::*;
use std::fs::File;
use std::io::Read;

static AFCONVERT_AAC_FILE: &str = "tests/afconvert-aac-0.5s.mp4";

extern "C" fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut File = unsafe { &mut *(userdata as *mut _) };
    let buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[test]
fn test_afconvert_aac_duration() {
    unsafe {
        let mut file =
            File::open(AFCONVERT_AAC_FILE).expect("Failed to open afconvert AAC test file");
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

        // The file should be exactly 0.5s long (22050 samples at 44100Hz = 0.5s)
        // afconvert produces files with elst box having start_time=2112 and segment_duration=22050
        // We should report the media duration (0.5s) not the track duration
        assert_eq!(
            track_info.duration, 22050,
            "Track duration should be 22050 (from elst segment_duration), got {}",
            track_info.duration
        );

        // Verify the duration represents 0.5 seconds
        let duration_seconds = track_info.duration as f64 / track_info.time_scale as f64;
        assert!(
            (duration_seconds - 0.5).abs() < 0.001,
            "Duration should be 0.5 seconds, got {} seconds",
            duration_seconds
        );

        mp4parse_free(parser);
    }
}
