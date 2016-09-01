extern crate mp4parse_capi;

use mp4parse_capi::*;

#[cfg(feature = "fuzz")]
#[macro_use]
extern crate abort_on_panic;

use std::io::Read;

extern fn vec_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let mut input: &mut std::io::Cursor<Vec<u8>> = unsafe { &mut *(userdata as *mut _) };

    let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(&mut buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

fn doit() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let mut cursor = std::io::Cursor::new(input);
    let io = mp4parse_io { read: vec_read, userdata: &mut cursor as *mut _ as *mut std::os::raw::c_void };
    unsafe {
        let context = mp4parse_new(&io);
        let rv = mp4parse_read(context);
        if rv == mp4parse_error::MP4PARSE_OK {
            let count = {
                let mut count = 0;
                let rv = mp4parse_get_track_count(context, &mut count);
                assert!(rv == mp4parse_error::MP4PARSE_OK);
                count
            };
            for track in 0..count {
                let mut info = mp4parse_track_info {
                    track_type: mp4parse_track_type::MP4PARSE_TRACK_TYPE_VIDEO,
                    codec: mp4parse_codec::MP4PARSE_CODEC_UNKNOWN,
                    track_id: 0,
                    duration: 0,
                    media_time: 0,
                };
                let rv = mp4parse_get_track_info(context, track, &mut info);
                if rv == mp4parse_error::MP4PARSE_OK {
                    println!("track {}: id={} duration={} media_time={}",
                             track, info.track_id, info.duration, info.media_time);
                    match info.track_type {
                        mp4parse_track_type::MP4PARSE_TRACK_TYPE_VIDEO => {
                            let mut video = mp4parse_track_video_info {
                                display_width: 0,
                                display_height: 0,
                                image_width: 0,
                                image_height: 0,
                            };
                            let rv = mp4parse_get_track_video_info(context, track, &mut video);
                            if rv == mp4parse_error::MP4PARSE_OK {
                                println!("  video: display={}x{} image={}x{}",
                                         video.display_width, video.display_height,
                                         video.image_width, video.image_height);
                            }
                        }
                        mp4parse_track_type::MP4PARSE_TRACK_TYPE_AUDIO => {
                            let mut audio = Default::default();
                            let rv = mp4parse_get_track_audio_info(context, track, &mut audio);
                            if rv == mp4parse_error::MP4PARSE_OK {
                                println!("  audio: channels={} bit_depth={} sample_rate={}",
                                         audio.channels, audio.bit_depth, audio.sample_rate);
                            }
                        }
                    }
                }
            }
        }
        mp4parse_free(context);
    }
}

#[cfg(feature = "fuzz")]
fn main() {
    abort_on_panic!({
        doit();
    });
}

#[cfg(not(feature = "fuzz"))]
fn main() {
    doit();
}
