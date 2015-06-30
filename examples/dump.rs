extern crate mp4parse;

use std::env;
use std::fs::File;

extern crate byteorder;
use byteorder::Error;

fn dump_file(filename: &String) {
    let mut f = File::open(filename).unwrap();
    loop {
        match mp4parse::read_box(&mut f) {
            Ok(_) => {},
            Err(Error::UnexpectedEOF) => { break },
            Err(e) => { panic!(e); },
        }
    }
}

fn main() {
    for filename in env::args().skip(1) {
        println!("-- dump of '{}' --", filename);
        dump_file(&filename);
        println!("-- end of '{}' --", filename);
    }
}
