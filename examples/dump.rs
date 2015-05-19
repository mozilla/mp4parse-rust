extern crate mp4parse;

use std::fs::File;
use std::io::{Seek,SeekFrom};

fn main() {
    let mut f = File::open("test.mp4").unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
    f.seek(SeekFrom::Current(h.size as i64 - 8)).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
}
