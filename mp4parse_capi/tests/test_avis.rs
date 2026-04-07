use mp4parse::unstable::Indice;
use mp4parse_capi::*;
use num_traits::ToPrimitive;
use std::io::Read;

extern "C" fn buf_read(buf: *mut u8, size: usize, userdata: *mut std::os::raw::c_void) -> isize {
    let input: &mut std::fs::File = unsafe { &mut *(userdata as *mut _) };
    let buf = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    match input.read(buf) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

fn default_avif_info() -> Mp4parseAvifInfo {
    Mp4parseAvifInfo {
        premultiplied_alpha: Default::default(),
        major_brand: Default::default(),
        unsupported_features_bitfield: Default::default(),
        spatial_extents: std::ptr::null(),
        nclx_colour_information: std::ptr::null(),
        icc_colour_information: Default::default(),
        image_rotation: mp4parse::ImageRotation::D0,
        image_mirror: std::ptr::null(),
        pixel_aspect_ratio: std::ptr::null(),
        has_primary_item: Default::default(),
        primary_item_bit_depth: Default::default(),
        has_alpha_item: Default::default(),
        alpha_item_bit_depth: Default::default(),
        has_sequence: Default::default(),
        loop_mode: Default::default(),
        loop_count: Default::default(),
        color_track_id: Default::default(),
        color_track_bit_depth: Default::default(),
        alpha_track_id: Default::default(),
        alpha_track_bit_depth: Default::default(),
    }
}

fn default_avif_image() -> Mp4parseAvifImage {
    Mp4parseAvifImage {
        primary_image: Default::default(),
        alpha_image: Default::default(),
    }
}

unsafe fn parse_file(path: &str) -> *mut Mp4parseAvifParser {
    let mut file = std::fs::File::open(path).expect("Unknown file");
    let io = Mp4parseIo {
        read: Some(buf_read),
        userdata: &mut file as *mut _ as *mut std::os::raw::c_void,
    };

    let mut parser = std::ptr::null_mut();
    let rv = mp4parse_avif_new(&io, ParseStrictness::Normal, &mut parser);
    assert_eq!(rv, Mp4parseStatus::Ok);
    assert!(!parser.is_null());

    parser
}

unsafe fn parse_file_and_get_info(path: &str) -> (*mut Mp4parseAvifParser, Mp4parseAvifInfo) {
    let parser = parse_file(path);
    let mut info = default_avif_info();
    let rv = mp4parse_avif_get_info(parser, &mut info);
    assert_eq!(rv, Mp4parseStatus::Ok);
    (parser, info)
}

unsafe fn assert_slice_pointer_is_readable<T>(ptr: *const T, len: usize) {
    let slice = std::slice::from_raw_parts(ptr, len);
    assert_eq!(slice.len(), len);
}

fn check_loop_count(path: &str, expected_loop_count: i64) {
    let (parser, info) = unsafe { parse_file_and_get_info(path) };
    match info.loop_mode {
        Mp4parseAvifLoopMode::NoEdits => assert_eq!(expected_loop_count, -1),
        Mp4parseAvifLoopMode::LoopByCount => {
            assert_eq!(info.loop_count.to_i64(), Some(expected_loop_count))
        }
        Mp4parseAvifLoopMode::LoopInfinitely => assert_eq!(expected_loop_count, i64::MIN),
    }

    unsafe { mp4parse_avif_free(parser) };
}

fn check_timescale(path: &str, expected_timescale: u64) {
    let (parser, info) = unsafe { parse_file_and_get_info(path) };

    let mut indices: Mp4parseByteData = Mp4parseByteData::default();
    let mut timescale: u64 = 0;
    let rv = unsafe {
        mp4parse_avif_get_indice_table(parser, info.color_track_id, &mut indices, &mut timescale)
    };

    assert_eq!(rv, Mp4parseStatus::Ok);
    assert_eq!(timescale, expected_timescale);

    unsafe { mp4parse_avif_free(parser) };
}

#[test]
fn loop_once() {
    check_loop_count("tests/loop_1.avif", 1);
}

#[test]
fn loop_twice() {
    check_loop_count("tests/loop_2.avif", 2);
}

#[test]
fn loop_four_times_due_to_ceiling() {
    check_loop_count("tests/loop_ceiled_4.avif", 4);
}

#[test]
fn loop_forever() {
    check_loop_count("tests/loop_forever.avif", i64::MIN);
}

#[test]
fn no_edts() {
    check_loop_count("tests/no_edts.avif", -1);
}

#[test]
fn check_timescales() {
    check_timescale("tests/loop_1.avif", 2);
    check_timescale("tests/loop_2.avif", 2);
    check_timescale("tests/loop_ceiled_4.avif", 2);
    check_timescale("tests/loop_forever.avif", 2);
    check_timescale("tests/no_edts.avif", 16384);
}

#[test]
fn repeated_get_info_returns_stable_pointers() {
    let (parser, info1) = unsafe { parse_file_and_get_info("tests/loop_1.avif") };

    unsafe {
        let mut info2 = default_avif_info();
        let rv = mp4parse_avif_get_info(parser, &mut info2);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(info1.spatial_extents, info2.spatial_extents);
        assert!(!info1.spatial_extents.is_null());
        assert_slice_pointer_is_readable(info1.spatial_extents, 1);

        assert_eq!(info1.nclx_colour_information, info2.nclx_colour_information);
        if !info1.nclx_colour_information.is_null() {
            assert_slice_pointer_is_readable(info1.nclx_colour_information, 1);
        }

        assert_eq!(
            info1.icc_colour_information.length,
            info2.icc_colour_information.length
        );
        assert_eq!(
            info1.icc_colour_information.data,
            info2.icc_colour_information.data
        );
        if info1.icc_colour_information.length == 0 {
            assert!(info1.icc_colour_information.data.is_null());
        } else {
            assert_slice_pointer_is_readable(
                info1.icc_colour_information.data,
                info1.icc_colour_information.length,
            );
        }

        assert_eq!(info1.image_mirror, info2.image_mirror);
        if !info1.image_mirror.is_null() {
            assert_slice_pointer_is_readable(info1.image_mirror, 1);
        }

        assert_eq!(info1.pixel_aspect_ratio, info2.pixel_aspect_ratio);
        if !info1.pixel_aspect_ratio.is_null() {
            assert_slice_pointer_is_readable(info1.pixel_aspect_ratio, 1);
        }

        mp4parse_avif_free(parser);
    }
}

#[test]
fn repeated_get_image_returns_stable_pointers() {
    let parser = unsafe { parse_file("tests/loop_1.avif") };

    unsafe {
        let mut image1 = default_avif_image();
        let rv = mp4parse_avif_get_image(parser, &mut image1);
        assert_eq!(rv, Mp4parseStatus::Ok);
        assert!(image1.primary_image.length > 0);
        assert!(!image1.primary_image.data.is_null());
        assert!(image1.alpha_image.length > 0);
        assert!(!image1.alpha_image.data.is_null());

        let mut image2 = default_avif_image();
        let rv = mp4parse_avif_get_image(parser, &mut image2);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(image1.primary_image.length, image2.primary_image.length);
        assert_eq!(image1.primary_image.data, image2.primary_image.data);
        assert_eq!(image1.alpha_image.length, image2.alpha_image.length);
        assert_eq!(image1.alpha_image.data, image2.alpha_image.data);

        assert_slice_pointer_is_readable(image1.primary_image.data, image1.primary_image.length);
        assert_slice_pointer_is_readable(image1.alpha_image.data, image1.alpha_image.length);

        mp4parse_avif_free(parser);
    }
}

#[test]
fn repeated_get_indice_table_returns_stable_pointer() {
    let (parser, info) = unsafe { parse_file_and_get_info("tests/loop_1.avif") };

    unsafe {
        let mut indices1 = Mp4parseByteData::default();
        let mut timescale1: u64 = 0;
        let rv = mp4parse_avif_get_indice_table(
            parser,
            info.color_track_id,
            &mut indices1,
            &mut timescale1,
        );
        assert_eq!(rv, Mp4parseStatus::Ok);
        assert!(indices1.length > 0);
        assert!(!indices1.indices.is_null());

        let mut indices2 = Mp4parseByteData::default();
        let mut timescale2: u64 = 0;
        let rv = mp4parse_avif_get_indice_table(
            parser,
            info.color_track_id,
            &mut indices2,
            &mut timescale2,
        );
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert_eq!(timescale1, timescale2);
        assert_eq!(indices1.length, indices2.length);
        assert_eq!(indices1.indices, indices2.indices);
        assert_slice_pointer_is_readable(indices1.indices, indices1.length);

        let first1: &[Indice] = std::slice::from_raw_parts(indices1.indices, indices1.length);
        let first2: &[Indice] = std::slice::from_raw_parts(indices2.indices, indices2.length);
        assert_eq!(first1[0], first2[0]);

        mp4parse_avif_free(parser);
    }
}

#[test]
fn empty_avif_byte_slices_use_null_pointers() {
    let (parser, info) = unsafe { parse_file_and_get_info("tests/no_edts.avif") };

    unsafe {
        let mut image = default_avif_image();
        let rv = mp4parse_avif_get_image(parser, &mut image);
        assert_eq!(rv, Mp4parseStatus::Ok);

        assert!(image.primary_image.length > 0);
        assert!(!image.primary_image.data.is_null());
        assert_eq!(image.alpha_image.length, 0);
        assert!(image.alpha_image.data.is_null());

        if info.icc_colour_information.length == 0 {
            assert!(info.icc_colour_information.data.is_null());
        }

        mp4parse_avif_free(parser);
    }
}
