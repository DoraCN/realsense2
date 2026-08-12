use std::ptr;
use std::slice;

use crate::error::Rs2Error;
use crate::ffi;
use crate::kind::{Rs2Extension, Rs2Format, Rs2StreamKind};
use crate::stream_profile::StreamProfile;

/// A single frame (or a composite frameset) returned by the pipeline.
///
/// Owns a C `rs2_frame` handle and releases it on drop via `rs2_release_frame`.
pub struct Frame {
    handle: *mut ffi::rs2_frame,
    owned: bool,
}

impl Frame {
    pub(crate) unsafe fn from_raw(handle: *mut ffi::rs2_frame) -> Frame {
        Frame {
            handle,
            owned: true,
        }
    }

    fn raw_data<'a>(&self) -> &'a [u8] {
        let mut err = ptr::null_mut();
        let data = unsafe { ffi::rs2_get_frame_data(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return &[];
        }
        if data.is_null() {
            return &[];
        }
        let size = self.data_size();
        unsafe { slice::from_raw_parts(data as *const u8, size) }
    }

    /// Raw frame bytes. Lifetime is tied to `&self`.
    pub fn data(&self) -> &[u8] {
        self.raw_data()
    }

    pub fn data_size(&self) -> usize {
        let mut err = ptr::null_mut();
        let n = unsafe { ffi::rs2_get_frame_data_size(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        n.max(0) as usize
    }

    pub fn width(&self) -> i32 {
        let mut err = ptr::null_mut();
        let w = unsafe { ffi::rs2_get_frame_width(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        w
    }

    pub fn height(&self) -> i32 {
        let mut err = ptr::null_mut();
        let h = unsafe { ffi::rs2_get_frame_height(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        h
    }

    pub fn stride_in_bytes(&self) -> i32 {
        let mut err = ptr::null_mut();
        let s = unsafe { ffi::rs2_get_frame_stride_in_bytes(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        s
    }

    pub fn bits_per_pixel(&self) -> i32 {
        let mut err = ptr::null_mut();
        let b = unsafe { ffi::rs2_get_frame_bits_per_pixel(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        b
    }

    pub fn timestamp_ms(&self) -> f64 {
        let mut err = ptr::null_mut();
        let t = unsafe { ffi::rs2_get_frame_timestamp(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0.0;
        }
        t
    }

    pub fn frame_number(&self) -> u64 {
        let mut err = ptr::null_mut();
        let n = unsafe { ffi::rs2_get_frame_number(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        n
    }

    /// The stream profile associated with this frame.
    pub fn stream_profile(&self) -> StreamProfile {
        let mut err = ptr::null_mut();
        let profile = unsafe { ffi::rs2_get_frame_stream_profile(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
        }
        // The profile is owned by the frame; wrap as a non-owning view.
        unsafe { StreamProfile::borrowed(profile) }
    }

    pub fn stream_kind(&self) -> Rs2StreamKind {
        self.stream_profile().stream_kind()
    }

    pub fn format(&self) -> Rs2Format {
        self.stream_profile().format()
    }

    /// Check whether this frame can be extended to the given extension type.
    pub fn is_extendable_to(&self, ext: Rs2Extension) -> bool {
        let mut err = ptr::null_mut();
        let r = unsafe { ffi::rs2_is_frame_extendable_to(self.handle, ext as i32, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return false;
        }
        r != 0
    }

    /// Depth units: meters represented by a single depth-count. Valid on depth
    /// frames; returns `None` otherwise.
    pub fn depth_units(&self) -> Option<f32> {
        if !self.is_extendable_to(Rs2Extension::DepthFrame) {
            return None;
        }
        let mut err = ptr::null_mut();
        let u = unsafe { ffi::rs2_depth_frame_get_units(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return None;
        }
        Some(u)
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            unsafe { ffi::rs2_release_frame(self.handle) };
        }
    }
}

/// A composite frameset: a frame containing multiple embedded frames, one per
/// active stream.
pub struct FrameSet {
    handle: *mut ffi::rs2_frame,
    owned: bool,
}

impl FrameSet {
    /// # Safety
    ///
    /// `handle` must be a valid composite frame whose ownership is transferred.
    pub(crate) unsafe fn from_raw(handle: *mut ffi::rs2_frame) -> FrameSet {
        FrameSet {
            handle,
            owned: true,
        }
    }
    /// Number of embedded frames (one per enabled stream).
    pub fn len(&self) -> usize {
        let mut err = ptr::null_mut();
        let n = unsafe { ffi::rs2_embedded_frames_count(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        n.max(0) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Extract the frame at `index`. The returned frame owns its own reference
    /// and may outlive this frameset; `rs2_extract_frame` returns a handle that
    /// must be released with `rs2_release_frame`, which [`Frame`] does on drop.
    pub fn extract(&self, index: usize) -> Option<Frame> {
        let mut err = ptr::null_mut();
        let frame_ptr = unsafe { ffi::rs2_extract_frame(self.handle, index as i32, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return None;
        }
        if frame_ptr.is_null() {
            return None;
        }
        Some(unsafe { Frame::from_raw(frame_ptr) })
    }

    /// Find the first embedded frame matching `stream`.
    pub fn frames_of_type(&self, stream: Rs2StreamKind) -> Vec<Frame> {
        let mut out = Vec::new();
        for i in 0..self.len() {
            if let Some(frame) = self.extract(i) {
                if frame.stream_kind() == stream {
                    out.push(frame);
                }
            }
        }
        out
    }
}

impl Drop for FrameSet {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            unsafe { ffi::rs2_release_frame(self.handle) };
        }
    }
}
