use std::sync::Arc;
use std::thread;

use super::*;

/// A `width`×`height` frame of one YCbCr colour, in `format`, rows padded to `pad` extra bytes.
fn solid(
    format: PixelFormat,
    width: usize,
    height: usize,
    ycbcr: [u8; 3],
    pad: usize,
) -> Vec<Vec<u8>> {
    let [y, cb, cr] = ycbcr;
    let (cw, ch) = (width.div_ceil(2), height.div_ceil(2));
    let plane = |row: Vec<u8>, rows: usize| -> Vec<u8> {
        let mut padded = row;
        padded.extend(std::iter::repeat_n(0xEE, pad));
        padded.repeat(rows)
    };
    match format {
        PixelFormat::Nv12 => vec![
            plane(vec![y; width], height),
            plane([cb, cr].repeat(cw), ch),
        ],
        PixelFormat::Nv21 => vec![
            plane(vec![y; width], height),
            plane([cr, cb].repeat(cw), ch),
        ],
        PixelFormat::I420 => vec![
            plane(vec![y; width], height),
            plane(vec![cb; cw], ch),
            plane(vec![cr; cw], ch),
        ],
    }
}

fn convert(
    format: PixelFormat,
    width: usize,
    height: usize,
    planes: &[Vec<u8>],
    strides: &[usize],
    matrix: Matrix,
) -> Result<Vec<u8>, FrameError> {
    let planes: Vec<Plane<'_>> = planes
        .iter()
        .zip(strides)
        .map(|(data, &stride)| Plane { data, stride })
        .collect();
    let mut out = Vec::new();
    to_rgba(
        &YuvFrame {
            format,
            width,
            height,
            planes: &planes,
        },
        matrix,
        &mut out,
    )?;
    Ok(out)
}

fn strides(format: PixelFormat, width: usize, pad: usize) -> Vec<usize> {
    match format {
        PixelFormat::Nv12 | PixelFormat::Nv21 => {
            vec![width + pad, width.div_ceil(2) * 2 + pad]
        }
        PixelFormat::I420 => vec![
            width + pad,
            width.div_ceil(2) + pad,
            width.div_ceil(2) + pad,
        ],
    }
}

fn near(actual: &[u8], expected: [u8; 4]) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, e)| (i32::from(*a) - i32::from(e)).abs() <= 2)
}

#[test]
fn reference_colours_convert_in_both_layouts_and_matrices() {
    // (Y, Cb, Cr) of limited-range colours and the RGB they stand for.
    let cases = [
        (Matrix::Bt601, [235, 128, 128], [255, 255, 255, 255]),
        (Matrix::Bt601, [16, 128, 128], [0, 0, 0, 255]),
        (Matrix::Bt601, [81, 90, 240], [255, 0, 0, 255]),
        (Matrix::Bt601, [145, 54, 34], [0, 255, 0, 255]),
        (Matrix::Bt601, [41, 240, 110], [0, 0, 255, 255]),
        (Matrix::Bt709, [63, 102, 240], [255, 0, 0, 255]),
        (Matrix::Bt709, [173, 42, 26], [0, 255, 0, 255]),
        (Matrix::Bt709, [32, 240, 118], [0, 0, 255, 255]),
    ];
    for format in [PixelFormat::Nv12, PixelFormat::Nv21, PixelFormat::I420] {
        for (matrix, ycbcr, rgba) in cases {
            let planes = solid(format, 4, 2, ycbcr, 0);
            let out = convert(format, 4, 2, &planes, &strides(format, 4, 0), matrix).unwrap();
            assert_eq!(out.len(), 4 * 2 * 4);
            for pixel in out.chunks(4) {
                assert!(
                    near(pixel, rgba),
                    "{format:?} {matrix:?} {ycbcr:?}: {pixel:?}"
                );
            }
        }
    }
}

#[test]
fn chroma_covers_two_by_two_pixels_and_strides_skip_padding() {
    // 4×4 NV12 with a different chroma sample per 2×2 block and padded rows.
    let width = 4;
    let pad = 3;
    let y: Vec<u8> = [vec![81; width], vec![0xEE; pad]].concat().repeat(4);
    let blocks = [[90u8, 240u8], [54, 34], [240, 110], [128, 128]];
    let mut uv = Vec::new();
    for row in 0..2 {
        for column in 0..2 {
            uv.extend_from_slice(&blocks[row * 2 + column]);
        }
        uv.extend(std::iter::repeat_n(0xEE, pad));
    }
    let out = convert(
        PixelFormat::Nv12,
        width,
        4,
        &[y, uv],
        &[width + pad, width + pad],
        Matrix::Bt601,
    )
    .unwrap();
    let at = |x: usize, y: usize| &out[(y * width + x) * 4..(y * width + x) * 4 + 4];
    // Each 2×2 block shares its chroma: the top-left block is red.
    for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        assert!(
            near(at(x, y), [255, 0, 0, 255]),
            "({x}, {y}): {:?}",
            at(x, y)
        );
    }
    assert_ne!(at(2, 0), at(0, 0), "the next block has its own chroma");
    assert_eq!(at(2, 0), at(3, 1));
    assert_eq!(at(0, 2), at(1, 3));

    // The same picture as I420 with padding gives the same pixels.
    for format in [PixelFormat::Nv12, PixelFormat::I420] {
        let planes = solid(format, 6, 4, [145, 54, 34], 5);
        let padded = convert(format, 6, 4, &planes, &strides(format, 6, 5), Matrix::Bt601).unwrap();
        let tight = convert(
            format,
            6,
            4,
            &solid(format, 6, 4, [145, 54, 34], 0),
            &strides(format, 6, 0),
            Matrix::Bt601,
        )
        .unwrap();
        assert_eq!(
            padded, tight,
            "{format:?}: padding never reaches the pixels"
        );
    }
}

#[test]
fn malformed_frames_are_refused() {
    let planes = solid(PixelFormat::Nv12, 4, 2, [16, 128, 128], 0);
    let nv12 = strides(PixelFormat::Nv12, 4, 0);
    assert_eq!(
        convert(PixelFormat::Nv12, 4, 0, &planes, &nv12, Matrix::Bt601),
        Err(FrameError::Size)
    );
    assert_eq!(
        convert(PixelFormat::I420, 4, 2, &planes, &nv12, Matrix::Bt601),
        Err(FrameError::Planes),
        "NV12 planes for an I420 frame"
    );
    let short = vec![planes[0].clone(), planes[1][..3].to_vec()];
    assert_eq!(
        convert(PixelFormat::Nv12, 4, 2, &short, &nv12, Matrix::Bt601),
        Err(FrameError::Truncated)
    );
    assert_eq!(
        convert(PixelFormat::Nv12, 4, 2, &planes, &[2, 4], Matrix::Bt601),
        Err(FrameError::Truncated),
        "a stride shorter than a row"
    );
}

#[test]
fn the_matrix_follows_the_frame_height() {
    assert_eq!(Matrix::for_height(480), Matrix::Bt601);
    assert_eq!(Matrix::for_height(576), Matrix::Bt601);
    assert_eq!(Matrix::for_height(720), Matrix::Bt709);
    assert_eq!(Matrix::for_height(2160), Matrix::Bt709);
}

fn frame(n: u8) -> RgbaFrame {
    RgbaFrame {
        width: 1,
        height: 1,
        pixels: vec![n, n, n, 255],
    }
}

#[test]
fn one_frame_waits_and_newer_ones_are_dropped_until_it_is_taken() {
    let slot = FrameSlot::default();
    assert!(slot.wants_frame());
    assert!(slot.offer(frame(1)));
    assert!(!slot.wants_frame(), "a frame is waiting: skip converting");
    assert!(!slot.offer(frame(2)), "a frame offered anyway is dropped");
    assert_eq!(slot.counts(), (1, 2));
    let shown = slot.take().unwrap();
    assert_eq!(shown, frame(1), "the waiting frame, not the newer one");
    assert_eq!(slot.take(), None);
    slot.recycle(shown.pixels);
    let buffer = slot.buffer();
    assert!(buffer.capacity() >= 4, "the buffer comes back for reuse");
    assert!(slot.buffer().is_empty(), "once");
    assert!(slot.wants_frame());
    assert!(slot.offer(frame(3)));
    assert_eq!(slot.take(), Some(frame(3)));
}

#[test]
fn a_decoder_thread_never_waits_for_the_main_thread() {
    let slot = Arc::new(FrameSlot::default());
    let decoder = {
        let slot = Arc::clone(&slot);
        thread::spawn(move || {
            for n in 0..200u8 {
                if slot.wants_frame() {
                    slot.offer(frame(n));
                }
            }
        })
    };
    let mut shown = Vec::new();
    while !decoder.is_finished() || shown.is_empty() {
        if let Some(frame) = slot.take() {
            shown.push(frame.pixels[0]);
        }
    }
    decoder.join().unwrap();
    if let Some(frame) = slot.take() {
        shown.push(frame.pixels[0]);
    }
    let (offered, dropped) = slot.counts();
    assert_eq!(
        offered as usize,
        shown.len(),
        "every offered frame is shown once"
    );
    assert_eq!(
        offered + dropped,
        200,
        "every decoded frame is shown or dropped"
    );
    assert!(
        shown.windows(2).all(|w| w[0] < w[1]),
        "in decode order: {shown:?}"
    );
}

#[test]
fn odd_sizes_take_a_chroma_sample_for_the_last_column_and_row() {
    for format in [PixelFormat::Nv12, PixelFormat::Nv21, PixelFormat::I420] {
        let planes = solid(format, 5, 3, [81, 90, 240], 1);
        let out = convert(format, 5, 3, &planes, &strides(format, 5, 1), Matrix::Bt601).unwrap();
        assert_eq!(out.len(), 5 * 3 * 4);
        for pixel in out.chunks(4) {
            assert!(near(pixel, [255, 0, 0, 255]), "{format:?}: {pixel:?}");
        }
    }
}
