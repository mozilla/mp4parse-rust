#![no_main]
use libfuzzer_sys::fuzz_target;

use mp4parse_capi::*;
use std::convert::TryInto;
use std::io::Read;

type CursorType<'a> = std::io::Cursor<&'a [u8]>;

extern "C" fn vec_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut CursorType = unsafe { &mut *(userdata as *mut _) };

    let buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(buf) {
        Ok(n) => n.try_into().expect("invalid conversion"),
        Err(_) => -1,
    }
}

fuzz_target!(|data: &[u8]| {
    let mut cursor: CursorType = std::io::Cursor::new(data);
    let io = Mp4parseIo {
        read: Some(vec_read),
        userdata: &mut cursor as *mut _ as *mut std::os::raw::c_void,
    };
    unsafe {
        let mut context = std::ptr::null_mut();
        if mp4parse_new(&io, &mut context) != Mp4parseStatus::Ok {
            return;
        }

        let mut frag_info = Default::default();
        mp4parse_get_fragment_info(context, &mut frag_info);

        let mut pssh_info = Default::default();
        mp4parse_get_pssh_info(context, &mut pssh_info);

        let mut count = 0;
        mp4parse_get_track_count(context, &mut count);

        for track in 0..count {
            let mut fragmented = 0;
            mp4parse_is_fragmented(context, track, &mut fragmented);

            let mut info = Default::default();
            mp4parse_get_track_info(context, track, &mut info);
            match info.track_type {
                Mp4parseTrackType::Video => {
                    let mut video = Mp4parseTrackVideoInfo::default();
                    mp4parse_get_track_video_info(context, track, &mut video);
                }
                Mp4parseTrackType::Audio => {
                    let mut audio = Default::default();
                    mp4parse_get_track_audio_info(context, track, &mut audio);
                }
                // No C API for metadata tracks yet.
                Mp4parseTrackType::Metadata => {}
            }

            let mut indices = Default::default();
            mp4parse_get_indice_table(context, track, &mut indices);
        }
        mp4parse_free(context);
    }
});
