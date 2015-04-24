// Module for parsing ISO Base Media Format aka video/mp4 streams.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Basic ISO box structure.
pub struct Mp4Box {
    /// Four character box type
    name: u32,
    /// Size of the box in bytes
    size: u64,
}

/// File type box 'ftyp'.
pub struct Mp4FileTypeBox {
    name: u32,
    size: u64,
    major_brand: u32,
    minor_version: u32,
    compatible_brands: Vec<u32>,
}

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

/// Parse a box out of a data buffer.
pub fn read_box<T: ReadBytesExt>(src: &mut T) -> Option<Mp4Box> {
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

/// Parse an ftype box.
pub fn read_ftype<T: ReadBytesExt>(src: &mut T) -> Option<Mp4FileTypeBox> {
    let head = read_box(src).unwrap();
    let major = src.read_u32::<BigEndian>().unwrap();
    let minor = src.read_u32::<BigEndian>().unwrap();
    let brand_count = (head.size - 8) /4;
//    let mut brands = Vec<u32>::with_capacity(brand_count);
    let mut brands = Vec::new();
    for _ in 0..brand_count {
        brands.push(src.read_u32::<BigEndian>().unwrap());
    }
    Some(Mp4FileTypeBox{
        name: head.name,
        size: head.size,
        major_brand: major,
        minor_version: minor,
        compatible_brands: brands,
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
fn test_read_box() {
    use std::io::Cursor;
    let mut test = "test".to_string().into_bytes();
    for x in [0, 0, 0, 8].iter() {
        test.push(*x);
    }
    let mut stream = Cursor::new(test);
    let parsed = read_box(&mut stream).unwrap();
    assert_eq!(parsed.name, 1952805748);
    assert_eq!(parsed.size, 8);
    println!("box {}", parsed);
}


#[test]
fn test_read_box_long() {
    use std::io::Cursor;
    let mut test = "long".to_string().into_bytes();
    for x in [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 16, 0].iter() {
        test.push(*x);
    }
    let mut stream = Cursor::new(test);
    let parsed = read_box(&mut stream).unwrap();
    assert_eq!(parsed.name, 1819242087);
    assert_eq!(parsed.size, 4096);
    println!("box {}", parsed);
}
