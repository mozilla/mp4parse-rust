extern crate mp4parse;

use std::env;
use std::fs::File;
use std::io::{Seek,SeekFrom};

fn dump_file(filename: String) {
    let mut f = File::open(filename).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
    f.seek(SeekFrom::Current((h.size - h.offset) as i64)).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
}

fn main() {
    for filename in env::args().skip(1) {
        println!("-- dump of '{}' --", filename);
        dump_file(filename);
    }
}
