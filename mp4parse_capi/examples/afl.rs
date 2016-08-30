extern crate mp4parse;

#[cfg(feature = "fuzz")]
#[macro_use]
extern crate abort_on_panic;

use std::io::{Cursor, Read};

fn doit() -> bool {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let mut reader = Cursor::new(&input);
    let mut context = mp4parse::MediaContext::new();
    return mp4parse::read_mp4(&mut reader, &mut context).is_ok();
}

#[cfg(feature = "fuzz")]
fn main() {
    abort_on_panic!({
        doit();
    });
}

#[cfg(not(feature = "fuzz"))]
fn main() {
    doit();
}
