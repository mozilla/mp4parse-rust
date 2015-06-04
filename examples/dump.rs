extern crate mp4parse;

use std::env;
use std::fs::File;
use std::io::{Read, Take};
use std::thread;

fn limit<'a>(f: &'a mut File, h: &mp4parse::BoxHeader) -> Take<&'a mut File> {
    f.take(h.size - h.offset)
}

fn read_box(f: &mut File) {
    mp4parse::read_box_header(f).and_then(|h| {
        match &(mp4parse::fourcc_to_string(h.name))[..] {
            "ftyp" => {
                let mut content = limit(f, &h);
                mp4parse::read_ftyp(&mut content, &h).and_then(|ftyp| {
                    println!("{}", ftyp);
                    Some(ftyp)
                })
            },
            _ => {
                println!("{}", h);
                mp4parse::skip_box_content(f, &h);
                None
            },
        }
    });
}

fn dump_file(filename: String) {
    let mut f = File::open(filename).unwrap();
    let task = thread::spawn(move || {
        loop {
            read_box(&mut f);
        }
    });
    // Catch and ignore any panics in the thread.
    task.join().ok();
}

fn main() {
    for filename in env::args().skip(1) {
        println!("-- dump of '{}' --", filename);
        dump_file(filename);
    }
}
