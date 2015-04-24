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

use std::fmt;
impl fmt::Display for Mp4Box {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_vec = |x| vec!((x >> 24 & 0xffu32) as u8,
                              (x >> 16 & 0xffu32) as u8,
                              (x >>  8 & 0xffu32) as u8,
                              (x & 0xffu32) as u8);
        let name_bytes = to_vec(self.name);
        let name = String::from_utf8_lossy(&name_bytes);
        write!(f, "'{}' {} bytes", name, self.size)
    }
}

#[test]
fn test_parse() {
    use std::io::Cursor;
    let test = vec!('t' as u8, 'e' as u8, 's' as u8, 't' as u8, 0, 0, 0, 8);
    let mut stream = Cursor::new(test);
    let parsed = read_box(&mut stream).unwrap();
    assert_eq!(parsed.name, 1952805748);
    assert_eq!(parsed.size, 8);
    println!("box {}", parsed);
}
