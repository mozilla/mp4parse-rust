extern crate mp4parse;

use std::env;
use std::fs::File;
use std::thread;

fn dump_file(filename: String) {
    let mut f = File::open(filename).unwrap();
    let task = thread::spawn(move || {
        loop {
            match mp4parse::read_box_header(&mut f) {
                Some(h) => {
                    println!("{}", h);
                    mp4parse::skip_box_content(&mut f, &h).unwrap();
                },
                _ => break,
            }
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
