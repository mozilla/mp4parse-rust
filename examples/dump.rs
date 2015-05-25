extern crate mp4parse;

use std::env;
use std::fs::File;

fn dump_file(filename: String) {
    let mut f = File::open(filename).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
    mp4parse::skip_box_content(&mut f, &h).unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
}

fn main() {
    for filename in env::args().skip(1) {
        println!("-- dump of '{}' --", filename);
        dump_file(filename);
    }
}
