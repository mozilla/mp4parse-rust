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
fn parse_rotation() {
    let mut file = std::fs::File::open("tests/video_rotation_90.mp4").expect("Unknown file");
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

        let mut video = mp4parse_track_video_info {
            display_width: 0,
            display_height: 0,
            image_width: 0,
            image_height: 0,
            rotation: 0,
            extra_data: mp4parse_byte_data::default(), 
            protected_data: Default::default(),
        };

        let rv = mp4parse_get_track_video_info(parser, 0, &mut video);
        assert_eq!(rv, mp4parse_status::OK);
        assert_eq!(video.rotation, 90);

        mp4parse_free(parser);
    }
}
