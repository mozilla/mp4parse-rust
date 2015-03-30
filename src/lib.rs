// Module for parsing ISO Base Media Format aka video/mp4 streams.

#![feature(old_io)] // for read_be_*()
use std::old_io::Reader;

/// Basic ISO box structure.
struct Mp4Box {
    /// Four character box type
    name: u32,
    /// Size of the box in bytes
    size: u64,
}

fn read_box(src: &mut Reader) -> Option<Mp4Box> {
    let name = src.read_be_u32().unwrap();
    let tmp_size = src.read_be_u32().unwrap();
    let size = match tmp_size {
        1 => src.read_be_u64().unwrap(),
        _ => tmp_size as u64,
    };
    Some(Mp4Box{
        name: name,
        size: size,
    })
}

#[test]
fn it_works() {
}
