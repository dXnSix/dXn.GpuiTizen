//! Decoded video frames for the texture output (`VID-3`, design `d25-video`): the player hands each
//! decoded frame (`player_set_media_packet_video_frame_decoded_cb`) to a player thread, which
//! converts it to RGBA here and offers it to the main thread through a one-frame [`FrameSlot`].
//!
//! - NV12 and NV21 (a Y plane and an interleaved CbCr or CrCb plane) and I420 (Y, Cb and Cr
//!   planes), all 4:2:0, limited range, BT.601 below 720 lines and BT.709 from 720 lines on, as
//!   decoders tag them by default; planes may have row padding (strides), and odd sizes have a
//!   chroma sample for the last, half-covered column or row.
//! - While a frame waits for the main thread, newer frames are dropped before they are converted:
//!   the main thread shows the frame it can, and the decoder is never held up.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

/// The decoded frame layouts the player hands over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// Y, then CbCr interleaved, each chroma sample covering 2×2 pixels.
    Nv12,
    /// Y, then CrCb interleaved.
    Nv21,
    /// Y, Cb, Cr, each chroma sample covering 2×2 pixels.
    I420,
}

/// One plane of a decoded frame: rows `stride` bytes apart.
#[derive(Clone, Copy, Debug)]
pub struct Plane<'a> {
    pub data: &'a [u8],
    pub stride: usize,
}

/// A decoded frame as the player's media packet describes it.
#[derive(Clone, Copy, Debug)]
pub struct YuvFrame<'a> {
    pub format: PixelFormat,
    pub width: usize,
    pub height: usize,
    /// NV12: Y and CbCr; NV21: Y and CrCb; I420: Y, Cb and Cr.
    pub planes: &'a [Plane<'a>],
}

/// The YCbCr to RGB conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matrix {
    Bt601,
    Bt709,
}

impl Matrix {
    /// BT.709 for HD frames (720 lines and more), BT.601 below.
    pub fn for_height(height: usize) -> Self {
        if height >= 720 {
            Self::Bt709
        } else {
            Self::Bt601
        }
    }

    /// Fixed-point (×256) coefficients: Cr→R, Cb→G, Cr→G, Cb→B.
    fn coefficients(self) -> [i32; 4] {
        match self {
            Self::Bt601 => [409, 100, 208, 516],
            Self::Bt709 => [459, 55, 136, 541],
        }
    }
}

/// Why a frame cannot be converted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// Zero dimensions.
    Size,
    /// Not the planes the format has.
    Planes,
    /// A plane shorter than its rows need.
    Truncated,
}

/// An RGBA frame for the main thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    /// `width × height × 4` bytes, rows top to bottom, no padding.
    pub pixels: Vec<u8>,
}

fn clamp(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

fn check(plane: &Plane<'_>, rows: usize, row_bytes: usize) -> Result<(), FrameError> {
    if plane.stride < row_bytes || plane.data.len() < plane.stride * (rows - 1) + row_bytes {
        Err(FrameError::Truncated)
    } else {
        Ok(())
    }
}

/// Converts `frame` to RGBA into `out` (resized; its allocation is reused).
pub fn to_rgba(frame: &YuvFrame<'_>, matrix: Matrix, out: &mut Vec<u8>) -> Result<(), FrameError> {
    let (width, height) = (frame.width, frame.height);
    if width == 0 || height == 0 {
        return Err(FrameError::Size);
    }
    let (chroma_width, chroma_height) = (width.div_ceil(2), height.div_ceil(2));
    let y = frame.planes.first().ok_or(FrameError::Planes)?;
    check(y, height, width)?;
    match frame.format {
        PixelFormat::Nv12 | PixelFormat::Nv21 => {
            let [_, uv] = frame.planes else {
                return Err(FrameError::Planes);
            };
            check(uv, chroma_height, chroma_width * 2)?;
        }
        PixelFormat::I420 => {
            let [_, u, v] = frame.planes else {
                return Err(FrameError::Planes);
            };
            check(u, chroma_height, chroma_width)?;
            check(v, chroma_height, chroma_width)?;
        }
    }
    let [cr_r, cb_g, cr_g, cb_b] = matrix.coefficients();
    out.resize(width * height * 4, 0);
    for row in 0..height {
        let y_row = &y.data[row * y.stride..row * y.stride + width];
        let chroma_row = row / 2;
        let out_row = &mut out[row * width * 4..(row + 1) * width * 4];
        for column in 0..width {
            let (cb, cr) = match frame.format {
                PixelFormat::Nv12 => {
                    let uv = &frame.planes[1];
                    let at = chroma_row * uv.stride + (column / 2) * 2;
                    (uv.data[at], uv.data[at + 1])
                }
                PixelFormat::Nv21 => {
                    let vu = &frame.planes[1];
                    let at = chroma_row * vu.stride + (column / 2) * 2;
                    (vu.data[at + 1], vu.data[at])
                }
                PixelFormat::I420 => {
                    let (u, v) = (&frame.planes[1], &frame.planes[2]);
                    (
                        u.data[chroma_row * u.stride + column / 2],
                        v.data[chroma_row * v.stride + column / 2],
                    )
                }
            };
            let luma = 298 * (i32::from(y_row[column]) - 16);
            let (cb, cr) = (i32::from(cb) - 128, i32::from(cr) - 128);
            let pixel = &mut out_row[column * 4..column * 4 + 4];
            pixel[0] = clamp((luma + cr_r * cr + 128) >> 8);
            pixel[1] = clamp((luma - cb_g * cb - cr_g * cr + 128) >> 8);
            pixel[2] = clamp((luma + cb_b * cb + 128) >> 8);
            pixel[3] = 255;
        }
    }
    Ok(())
}

/// One frame waiting for the main thread, and buffers to reuse.
///
/// The player thread asks [`FrameSlot::wants_frame`] before converting, converts into
/// [`FrameSlot::buffer`] and [`FrameSlot::offer`]s the result; the main thread
/// [`FrameSlot::take`]s it, uploads it and gives the buffer back with [`FrameSlot::recycle`].
#[derive(Debug, Default)]
pub struct FrameSlot {
    pending: Mutex<Option<RgbaFrame>>,
    spare: Mutex<Option<Vec<u8>>>,
    offered: AtomicU64,
    dropped: AtomicU64,
}

impl FrameSlot {
    /// Whether a new frame would be shown: no frame is waiting. Counts a drop when one is.
    pub fn wants_frame(&self) -> bool {
        let waiting = self
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some();
        if waiting {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        !waiting
    }

    /// A buffer to convert the next frame into.
    pub fn buffer(&self) -> Vec<u8> {
        self.spare
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .unwrap_or_default()
    }

    /// Offers a converted frame; returns false (and drops it) when one is still waiting.
    pub fn offer(&self, frame: RgbaFrame) -> bool {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        if pending.is_some() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        *pending = Some(frame);
        self.offered.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// The waiting frame, if any (main thread).
    pub fn take(&self) -> Option<RgbaFrame> {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }

    /// Gives a frame's buffer back for the next conversion.
    pub fn recycle(&self, pixels: Vec<u8>) {
        *self.spare.lock().unwrap_or_else(PoisonError::into_inner) = Some(pixels);
    }

    /// Frames offered and frames dropped so far.
    pub fn counts(&self) -> (u64, u64) {
        (
            self.offered.load(Ordering::Relaxed),
            self.dropped.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests;
