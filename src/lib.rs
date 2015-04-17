// Module for parsing ISO Base Media Format aka video/mp4 streams.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Basic ISO box structure.
struct Mp4Box {
    /// Four character box type
    name: u32,
    /// Size of the box in bytes
    size: u64,
}

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

/// Parse a box out of a data buffer.
fn read_box<T: ReadBytesExt>(src: &mut T) -> Option<Mp4Box> {
    let name = src.read_u32::<BigEndian>().unwrap();
    let tmp_size = src.read_u32::<BigEndian>().unwrap();
    let size = match tmp_size {
        1 => src.read_u64::<BigEndian>().unwrap(),
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
