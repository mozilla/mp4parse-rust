use mp4parse_capi::*;
use std::io::Read;

extern "C" fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };
    let buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

unsafe fn open_parser(path: &str) -> *mut Mp4parseParser {
    let mut file = std::fs::File::open(path).expect("file not found");
    let io = Mp4parseIo {
        read: Some(buf_read),
        userdata: &mut file as *mut _ as *mut std::os::raw::c_void,
    };
    let mut parser = std::ptr::null_mut();
    let rv = mp4parse_new(&io, &mut parser);
    assert_eq!(rv, Mp4parseStatus::Ok);
    assert!(!parser.is_null());
    parser
}

/// HDR10 (PQ): colour_primaries=9, transfer=16, matrix=9, full_range=false
#[test]
fn video_colr_nclx_hdr10() {
    unsafe {
        let parser = open_parser("tests/video_colr_nclx_hdr10.mp4");

        let mut video = Mp4parseTrackVideoInfo::default();
        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(video.sample_info_count, 1);
        let sample = &*video.sample_info;
        assert!(sample.has_colour_info);
        assert_eq!(sample.colour_primaries, 9);
        assert_eq!(sample.transfer_characteristics, 16);
        assert_eq!(sample.matrix_coefficients, 9);
        assert!(!sample.full_range_flag);

        mp4parse_free(parser);
    }
}

/// HDR10 full-range: colour_primaries=9, transfer=16, matrix=9, full_range=true
#[test]
fn video_colr_nclx_hdr10_full_range() {
    unsafe {
        let parser = open_parser("tests/video_colr_nclx_hdr10_full_range.mp4");

        let mut video = Mp4parseTrackVideoInfo::default();
        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(video.sample_info_count, 1);
        let sample = &*video.sample_info;
        assert!(sample.has_colour_info);
        assert_eq!(sample.colour_primaries, 9);
        assert_eq!(sample.transfer_characteristics, 16);
        assert_eq!(sample.matrix_coefficients, 9);
        assert!(sample.full_range_flag);

        mp4parse_free(parser);
    }
}

/// HLG: colour_primaries=9, transfer=18, matrix=9, full_range=false
#[test]
fn video_colr_nclx_hlg() {
    unsafe {
        let parser = open_parser("tests/video_colr_nclx_hlg.mp4");

        let mut video = Mp4parseTrackVideoInfo::default();
        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(video.sample_info_count, 1);
        let sample = &*video.sample_info;
        assert!(sample.has_colour_info);
        assert_eq!(sample.colour_primaries, 9);
        assert_eq!(sample.transfer_characteristics, 18);
        assert_eq!(sample.matrix_coefficients, 9);
        assert!(!sample.full_range_flag);

        mp4parse_free(parser);
    }
}

/// HLG full-range: colour_primaries=9, transfer=18, matrix=9, full_range=true
#[test]
fn video_colr_nclx_hlg_full_range() {
    unsafe {
        let parser = open_parser("tests/video_colr_nclx_hlg_full_range.mp4");

        let mut video = Mp4parseTrackVideoInfo::default();
        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(video.sample_info_count, 1);
        let sample = &*video.sample_info;
        assert!(sample.has_colour_info);
        assert_eq!(sample.colour_primaries, 9);
        assert_eq!(sample.transfer_characteristics, 18);
        assert_eq!(sample.matrix_coefficients, 9);
        assert!(sample.full_range_flag);

        mp4parse_free(parser);
    }
}

/// RGB (Identity matrix, MC=0): has_colour_info=true but matrix_coefficients=0.
/// Validates that has_colour_info correctly distinguishes "colr box present with MC=0"
/// from "no colr box" (where matrix_coefficients is also 0).
#[test]
fn video_colr_nclx_rgb_identity_matrix() {
    unsafe {
        let parser = open_parser("tests/video_colr_nclx_rgb.mp4");

        let mut video = Mp4parseTrackVideoInfo::default();
        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(video.sample_info_count, 1);
        let sample = &*video.sample_info;
        assert!(sample.has_colour_info);
        assert_eq!(sample.colour_primaries, 1);
        assert_eq!(sample.transfer_characteristics, 1);
        assert_eq!(sample.matrix_coefficients, 0); // Identity/GBR — valid CICP value, not "absent"
        assert!(!sample.full_range_flag);

        mp4parse_free(parser);
    }
}

/// No colr box: has_colour_info=false, CICP fields are zero
#[test]
fn video_no_colr_box() {
    unsafe {
        let parser = open_parser("tests/white.mp4");

        let mut video = Mp4parseTrackVideoInfo::default();
        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(video.sample_info_count, 1);
        let sample = &*video.sample_info;
        assert!(!sample.has_colour_info);
        assert_eq!(sample.colour_primaries, 0);
        assert_eq!(sample.transfer_characteristics, 0);
        assert_eq!(sample.matrix_coefficients, 0);
        assert!(!sample.full_range_flag);

        mp4parse_free(parser);
    }
}
