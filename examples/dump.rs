extern crate mp4parse;

use std::env;
use std::fs::File;
use std::io::{Read, Seek, Take};
use std::io::Cursor;
use std::thread;

fn limit<'a, T: Read>(f: &'a mut T, h: &mp4parse::BoxHeader) -> Take<&'a mut T> {
    f.take(h.size - h.offset)
}

fn recurse<T: Read>(f: &mut T, h: &mp4parse::BoxHeader) {
    println!("{} -- recursing", h);
    let buf: Vec<u8> = limit(f, &h)
        .bytes()
        .map(|u| u.unwrap())
        .collect();
    let mut content = Cursor::new(buf);
    loop {
        read_box(&mut content);
    }
    println!("{} -- end", h);
}

fn read_box<T: Read + Seek>(f: &mut T) {
    mp4parse::read_box_header(f).and_then(|h| {
        match &(mp4parse::fourcc_to_string(h.name))[..] {
            "ftyp" => {
                let mut content = limit(f, &h);
                let ftyp = mp4parse::read_ftyp(&mut content, &h).unwrap();
                println!("{}", ftyp);
            },
            "moov" => recurse(f, &h),
            "mvhd" => {
                let mut content = limit(f, &h);
                let mvhd = mp4parse::read_mvhd(&mut content, &h).unwrap();
                println!("  {}", mvhd);
            },
            "trak" => recurse(f, &h),
            _ => {
                // Skip the contents of unknown chunks.
                println!("{}", h);
                mp4parse::skip_box_content(f, &h).unwrap();
            },
        };
        Some(()) // and_then needs a Option.
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
