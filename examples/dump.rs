extern crate mp4parse;

use std::env;
use std::fs::File;
use std::io::BufReader;

fn dump_file(filename: &String) {
    let file = File::open(filename).unwrap();
    let mut reader = BufReader::new(file);
    let mut context = mp4parse::MediaContext::new();
    // Turn on debug output.
    context.trace(true);
    // Read all boxes.
    match mp4parse::read_mp4(&mut reader, &mut context) {
        Ok(_) => {},
        Err(mp4parse::Error::UnexpectedEOF) => {},
        Err(mp4parse::Error::Io(e)) => {
            println!("I/O ERROR: {:?}", e);
            panic!(e);
        },
        Err(e) => {
            println!("ERROR: {:?}", e);
        },
    }
    println!("-- result of parsing '{}' --\n{:?}", filename, context);
}

fn main() {
    for filename in env::args().skip(1) {
        println!("-- dump of '{}' --", filename);
        dump_file(&filename);
        println!("-- end of '{}' --", filename);
    }
}
