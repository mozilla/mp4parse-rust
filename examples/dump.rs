extern crate mp4parse;

use std::env;
use std::fs::File;
use std::io::BufReader;

extern crate byteorder;
use byteorder::Error;

fn dump_file(filename: &String) {
    let file = File::open(filename).unwrap();
    let mut reader = BufReader::new(file);
    loop {
        let mut context = mp4parse::MediaContext { tracks: Vec::new() };
        match mp4parse::read_box(&mut reader, &mut context) {
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
