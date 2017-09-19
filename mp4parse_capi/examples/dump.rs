extern crate mp4parse;
extern crate mp4parse_capi;

use std::env;
use std::fs::File;
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

fn dump_file(filename: &str, verbose: bool) {
    let mut file = File::open(filename).expect("Unknown file");
    let io = mp4parse_io {
        read: Some(buf_read),
        userdata: &mut file as *mut _ as *mut std::os::raw::c_void
    };

    unsafe {
        let parser = mp4parse_new(&io);

        if verbose {
            mp4parse_log(true);
        }

        match mp4parse_read(parser) {
            mp4parse_status::OK => (),
            _ => {
                println!("-- fail to parse, '-v' for more info");
                return;
            },
        }

        let mut frag_info = mp4parse_fragment_info { .. Default::default() };
        match mp4parse_get_fragment_info(parser, &mut frag_info) {
            mp4parse_status::OK => {
                println!("-- mp4parse_fragment_info {:?}", frag_info);
            },
            _ => {
                println!("-- mp4parse_fragment_info failed");
                return;
            }
        }

        let mut counts: u32 = 0;
        match mp4parse_get_track_count(parser, &mut counts) {
            mp4parse_status::OK => (),
            _ => {
                println!("-- mp4parse_get_track_count failed");
                return;
            }
        }

        for i in 0 .. counts {
            let mut track_info = mp4parse_track_info {
                track_type: mp4parse_track_type::AUDIO,
                codec: mp4parse_codec::UNKNOWN,
                track_id: 0,
                duration: 0,
                media_time: 0,
            };
            match mp4parse_get_track_info(parser, i, &mut track_info) {
                mp4parse_status::OK => {
                    println!("-- mp4parse_get_track_info {:?}", track_info);
                },
                _ => {
                    println!("-- mp4parse_get_track_info failed, track id: {}", i);
                    return;
                }
            }

            match track_info.track_type {
                mp4parse_track_type::AUDIO => {
                    let mut audio_info = mp4parse_track_audio_info { .. Default::default() };
                    match mp4parse_get_track_audio_info(parser, i, &mut audio_info) {
                        mp4parse_status::OK => {
                          println!("-- mp4parse_get_track_audio_info {:?}", audio_info);
                        },
                        _ => {
                          println!("-- mp4parse_get_track_audio_info failed, track id: {}", i);
                          return;
                        }
                    }
                },
                mp4parse_track_type::VIDEO => {
                    let mut video_info = mp4parse_track_video_info { .. Default::default() };
                    match mp4parse_get_track_video_info(parser, i, &mut video_info) {
                        mp4parse_status::OK => {
                          println!("-- mp4parse_get_track_video_info {:?}", video_info);
                        },
                        _ => {
                          println!("-- mp4parse_get_track_video_info failed, track id: {}", i);
                          return;
                        }
                    }
                },
            }

            let mut indices = mp4parse_byte_data::default();
            match mp4parse_get_indice_table(parser, track_info.track_id, &mut indices) {
                mp4parse_status::OK => {
                  println!("-- mp4parse_get_indice_table track_id {} indices {:?}", track_info.track_id, indices);
                },
                _ => {
                  println!("-- mp4parse_get_indice_table failed, track_info.track_id: {}", track_info.track_id);
                  return;
                }
            }
        }
        mp4parse_free(parser);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    let (skip, verbose) = if args[1] == "-v" {
        (2, true)
    } else {
        (1, false)
    };
    for filename in args.iter().skip(skip) {
        if verbose {
            println!("-- dump of '{}' --", filename);
        }
        dump_file(filename, verbose);
        if verbose {
            println!("-- end of '{}' --", filename);
        }
    }
}
