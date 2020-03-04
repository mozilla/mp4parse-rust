#![no_main]
use libfuzzer_sys::fuzz_target;

use mp4parse_capi::*;
use std::convert::TryInto;
use std::io::Read;

type CursorType<'a> = std::io::Cursor<&'a [u8]>;

extern "C" fn vec_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut CursorType = unsafe { &mut *(userdata as *mut _) };

    let mut buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(&mut buf) {
        Ok(n) => n.try_into().expect("invalid conversion"),
        Err(_) => -1,
    }
}

fuzz_target!(|data: &[u8]| {
    let mut cursor: CursorType = std::io::Cursor::new(data);
    let io = Mp4parseIo {
        read: Some(vec_read),
        userdata: &mut cursor as *mut _ as *mut std::os::raw::c_void,
    };
    unsafe {
        let context = mp4parse_avif_new(&io);

        if mp4parse_avif_read(context) != Mp4parseStatus::Ok {
            mp4parse_avif_free(context);
            return;
        }

        let mut primary_item = Default::default();
        mp4parse_avif_get_primary_item(context, &mut primary_item);

        mp4parse_avif_free(context);
    }
});
