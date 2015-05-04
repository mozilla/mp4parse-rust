extern crate mp4parse;

use std::fs::File;

fn main() {
    let mut f = File::open("test.mp4").unwrap();
    let h = mp4parse::read_box_header(&mut f).unwrap();
    println!("{}", h);
}
