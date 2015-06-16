extern crate mp4parse;

use std::env;
use std::fs::File;

fn dump_file(filename: String) {
    let mut f = File::open(filename).unwrap();
    loop {
        mp4parse::read_box(&mut f).unwrap();
    }
}

fn main() {
    for filename in env::args().skip(1) {
        println!("-- dump of '{}' --", filename);
        dump_file(filename);
    }
}
