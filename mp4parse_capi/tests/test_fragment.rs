extern crate mp4parse_capi;
use std::io::Read;
use mp4parse_capi::*;

extern fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };
    let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(&mut buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[test]
fn parse_fragment() {
    let mut file = std::fs::File::open("tests/bipbop_audioinit.mp4").expect("Unknown file");
    let io = mp4parse_io {
        read: Some(buf_read),
        userdata: &mut file as *mut _ as *mut std::os::raw::c_void
    };

    unsafe {
        let parser = mp4parse_new(&io);

        let mut rv = mp4parse_read(parser);
        assert_eq!(rv, mp4parse_status::OK);

        let mut counts: u32 = 0;
        rv = mp4parse_get_track_count(parser, &mut counts);
        assert_eq!(rv, mp4parse_status::OK);
        assert_eq!(counts, 1);

        let mut track_info = mp4parse_track_info {
            track_type: mp4parse_track_type::AUDIO,
            codec: mp4parse_codec::UNKNOWN,
            track_id: 0,
            duration: 0,
            media_time: 0,
        };
        rv = mp4parse_get_track_info(parser, 0, &mut track_info);
        assert_eq!(rv, mp4parse_status::OK);
        assert_eq!(track_info.track_type, mp4parse_track_type::AUDIO);
        assert_eq!(track_info.codec, mp4parse_codec::AAC);
        assert_eq!(track_info.track_id, 1);
        assert_eq!(track_info.duration, 0);
        assert_eq!(track_info.media_time, 0);

        let mut is_fragmented_file: u8 = 0;
        rv = mp4parse_is_fragmented(parser, track_info.track_id, &mut is_fragmented_file);
        assert_eq!(rv, mp4parse_status::OK);
        assert_eq!(is_fragmented_file, 1);

        let mut fragment_info = mp4parse_fragment_info {
            fragment_duration: 0,
        };
        rv = mp4parse_get_fragment_info(parser, &mut fragment_info);
        assert_eq!(rv, mp4parse_status::OK);
        assert_eq!(fragment_info.fragment_duration, 10032000);

        mp4parse_free(parser);
    }
}
