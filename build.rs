#[cfg(feature = "cheddar")]
extern crate cheddar;

#[cfg(feature = "cheddar")]
fn main() {
    cheddar::Cheddar::new().expect("could not read manifest")
        .module("capi")
        .run_build("include/mp4parse.h");
}

#[cfg(not(feature = "cheddar"))]
fn main() {
}

