extern crate mp4parse;

use std::env;
use std::fs::File;
use std::io::BufReader;

fn dump_file(filename: &String, verbose: bool) {
    let file = match File::open(filename) {
        Ok(file) => file,
        _ => {
            println!("ERROR: invalid path '{}'", filename);
            return;
        }
    };
    let mut reader = BufReader::new(file);
    let mut context = mp4parse::MediaContext::new();
    // Turn on debug output.
    if verbose {
        context.trace(true);
    }
    // Read all boxes.
    match mp4parse::read_mp4(&mut reader, &mut context) {
        Ok(_) => {},
        Err(mp4parse::Error::Io(e)) => {
            println!("I/O ERROR: {:?} in '{}'", e, filename);
            panic!(e);
        },
        Err(e) => {
            println!("ERROR: {:?} in '{}'", e, filename);
        },
    }
    if verbose {
        println!("-- result of parsing '{}' --\n{:?}", filename, context);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    let (skip, verbose) = if args[1] == "-v" {
        (2, true)
    } else {
        (1, false)
    };
    for filename in args.iter().skip(skip) {
        if verbose {
            println!("-- dump of '{}' --", filename);
        }
        dump_file(&filename, verbose);
        if verbose {
            println!("-- end of '{}' --", filename);
        }
    }
}
