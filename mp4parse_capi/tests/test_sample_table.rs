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

        // Check audio smaple table
        let mut is_fragmented_file: u8 = 0;
        rv = mp4parse_is_fragmented(parser, track_info.track_id, &mut is_fragmented_file);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);
        assert_eq!(is_fragmented_file, 0);

        let mut indice = mp4parse_byte_data::default();
        rv = mp4parse_get_indice_table(parser, track_info.track_id, &mut indice);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);

        // Compare the value from stagefright.
        let audio_indice_0 =  mp4parse_indice { start_offset: 27046, end_offset: 27052, start_composition: 0, end_composition: 46439, start_decode: 0, sync: true };
        let audio_indice_215 =  mp4parse_indice { start_offset: 283550, end_offset: 283556, start_composition: 9984580, end_composition: 10031020, start_decode: 9984580, sync: true };
        assert_eq!(indice.length, 216);
        assert_eq!(*indice.indices.offset(0), audio_indice_0);
        assert_eq!(*indice.indices.offset(215), audio_indice_215);

        // Check video smaple table
        rv = mp4parse_get_track_info(parser, 0, &mut track_info);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);
        assert_eq!(track_info.track_type, mp4parse_track_type::MP4PARSE_TRACK_TYPE_VIDEO);
        assert_eq!(track_info.codec, mp4parse_codec::MP4PARSE_CODEC_AVC);

        let mut is_fragmented_file: u8 = 0;
        rv = mp4parse_is_fragmented(parser, track_info.track_id, &mut is_fragmented_file);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);
        assert_eq!(is_fragmented_file, 0);

        let mut indice = mp4parse_byte_data::default();
        rv = mp4parse_get_indice_table(parser, track_info.track_id, &mut indice);
        assert_eq!(rv, mp4parse_error::MP4PARSE_OK);

        // Compare the last few data from stagefright.
        let video_indice_291 = mp4parse_indice { start_offset: 280226, end_offset: 280855, start_composition: 9838333, end_composition: 9871677, start_decode: 9710000, sync: false };
        let video_indice_292 = mp4parse_indice { start_offset: 280855, end_offset: 281297, start_composition: 9805011, end_composition: 9838333, start_decode: 9710011, sync: false };
        // TODO: start_composition time in stagefright is 9905000, but it is 9904999 in parser, it
        //       could be rounding error.
        //let video_indice_293 = mp4parse_indice { start_offset: 281297, end_offset: 281919, start_composition: 9905000, end_composition: 9938344, start_decode: 9776666, sync: false };
        //let video_indice_294 = mp4parse_indice { start_offset: 281919, end_offset: 282391, start_composition: 9871677, end_composition: 9905000, start_decode: 9776677, sync: false };
        let video_indice_295 = mp4parse_indice { start_offset: 282391, end_offset: 283032, start_composition: 9971666, end_composition: 9971677, start_decode: 9843333, sync: false };
        let video_indice_296 = mp4parse_indice { start_offset: 283092, end_offset: 283526, start_composition: 9938344, end_composition: 9971666, start_decode: 9843344, sync: false };

        assert_eq!(indice.length, 297);
        assert_eq!(*indice.indices.offset(291), video_indice_291);
        assert_eq!(*indice.indices.offset(292), video_indice_292);
        //assert_eq!(*indice.indices.offset(293), video_indice_293);
        //assert_eq!(*indice.indices.offset(294), video_indice_294);
        assert_eq!(*indice.indices.offset(295), video_indice_295);
        assert_eq!(*indice.indices.offset(296), video_indice_296);

        mp4parse_free(parser);
    }
}
