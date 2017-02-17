extern crate mp4parse_capi;
use std::io::Read;
use mp4parse_capi::*;

extern fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let mut input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };
    let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(&mut buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[test]
fn parse_sample_table() {
    let mut file = std::fs::File::open("tests/bipbop_nonfragment_header.mp4").expect("Unknown file");
    let io = mp4parse_io {
        read: buf_read,
        userdata: &mut file as *mut _ as *mut std::os::raw::c_void
    };

    unsafe {
        let parser = mp4parse_new(&io);

        let mut rv = mp4parse_read(parser);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);

        let mut counts: u32 = 0;
        rv = mp4parse_get_track_count(parser, &mut counts);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);
        assert_eq!(counts, 2);

        let mut track_info = mp4parse_track_info {
            track_type: mp4parse_track_type::MP4PARSE_TRACK_TYPE_AUDIO,
            codec: mp4parse_codec::MP4PARSE_CODEC_UNKNOWN,
            track_id: 0,
            duration: 0,
            media_time: 0,
        };
        rv = mp4parse_get_track_info(parser, 1, &mut track_info);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);
        assert_eq!(track_info.track_type, mp4parse_track_type::MP4PARSE_TRACK_TYPE_AUDIO);
        assert_eq!(track_info.codec, mp4parse_codec::MP4PARSE_CODEC_AAC);

        let mut is_fragmented_file: u8 = 0;
        rv = mp4parse_is_fragmented(parser, track_info.track_id, &mut is_fragmented_file);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);
        assert_eq!(is_fragmented_file, 0);

        let mut indice = mp4parse_byte_data::default();
        rv = mp4parse_get_indice_table(parser, track_info.track_id, &mut indice);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);

        // Compare the value from stagefright.
        let first_indice =  mp4parse_indice { start_offset: 27046, end_offset: 27052, start_composition: 0, end_composition: 46439, start_decode: 0, sync: true };
        let last_indice =  mp4parse_indice { start_offset: 283550, end_offset: 283556, start_composition: 9984580, end_composition: 10031020, start_decode: 9984580, sync: true };
        assert_eq!(indice.length, 216);
        assert_eq!(*indice.indices.offset(0), first_indice);
        assert_eq!(*indice.indices.offset(215), last_indice);

        mp4parse_free(parser);
    }
}
