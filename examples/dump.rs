extern crate mp4parse;

use std::fs::File;
use std::io::{Seek,SeekFrom};

fn dump_file(filename: String) {
    let mut f = File::open(filename).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
    f.seek(SeekFrom::Current(h.size as i64 - 8)).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
}

fn main() {
    let filename = "test.mp4".to_string();
    dump_file(filename);
}
