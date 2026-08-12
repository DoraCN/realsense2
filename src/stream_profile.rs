use std::ptr;

use crate::error::Rs2Error;
use crate::ffi;
use crate::kind::{Rs2Format, Rs2StreamKind};

/// A description of a stream configuration (resolution, format, framerate).
pub struct StreamProfile {
    handle: *const ffi::rs2_stream_profile,
    owned: bool,
}

impl StreamProfile {
    /// # Safety
    ///
    /// `handle` must be valid for the lifetime of the returned object.
    pub(crate) unsafe fn borrowed(handle: *const ffi::rs2_stream_profile) -> StreamProfile {
        StreamProfile {
            handle,
            owned: false,
        }
    }

    pub(crate) fn from_owned_list(list: &StreamProfileList, index: i32) -> StreamProfile {
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_get_stream_profile(list.handle, index, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
        }
        unsafe { StreamProfile::borrowed(handle) }
    }

    pub fn stream_kind(&self) -> Rs2StreamKind {
        let (stream, _, _, _, _, _) = self.data();
        stream
    }

    pub fn format(&self) -> Rs2Format {
        let (_, format, _, _, _, _) = self.data();
        format
    }

    pub fn index(&self) -> i32 {
        let (_, _, index, _, _, _) = self.data();
        index
    }

    pub fn unique_id(&self) -> i32 {
        let (_, _, _, unique_id, _, _) = self.data();
        unique_id
    }

    pub fn framerate(&self) -> i32 {
        let (_, _, _, _, framerate, _) = self.data();
        framerate
    }

    /// (stream_kind, format, index, unique_id, framerate, resolution)
    pub fn data(&self) -> (Rs2StreamKind, Rs2Format, i32, i32, i32, (i32, i32)) {
        let mut stream: i32 = 0;
        let mut format: i32 = 0;
        let mut index: i32 = 0;
        let mut unique_id: i32 = 0;
        let mut framerate: i32 = 0;
        let mut err = ptr::null_mut();
        unsafe {
            ffi::rs2_get_stream_profile_data(
                self.handle,
                &mut stream,
                &mut format,
                &mut index,
                &mut unique_id,
                &mut framerate,
                &mut err,
            )
        };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
        }
        let resolution = if self.is_video() {
            let mut width: i32 = 0;
            let mut height: i32 = 0;
            let mut res_err = ptr::null_mut();
            unsafe {
                ffi::rs2_get_video_stream_resolution(self.handle, &mut width, &mut height, &mut res_err)
            };
            if !res_err.is_null() {
                let _ = unsafe { Rs2Error::from_ptr(res_err) };
            }
            (width, height)
        } else {
            (0, 0)
        };
        (
            int_to_stream(stream),
            int_to_format(format),
            index,
            unique_id,
            framerate,
            resolution,
        )
    }

    pub fn is_video(&self) -> bool {
        matches!(
            self.stream_kind(),
            Rs2StreamKind::Depth
                | Rs2StreamKind::Color
                | Rs2StreamKind::Infrared
                | Rs2StreamKind::Fisheye
        )
    }
}

impl Drop for StreamProfile {
    fn drop(&mut self) {
        // Stream profiles obtained from a list or frame are owned by their
        // parent; only cloned profiles need rs2_delete_stream_profile.
        let _ = &self.owned;
    }
}

fn int_to_stream(v: i32) -> Rs2StreamKind {
    match v {
        1 => Rs2StreamKind::Depth,
        2 => Rs2StreamKind::Color,
        3 => Rs2StreamKind::Infrared,
        4 => Rs2StreamKind::Fisheye,
        5 => Rs2StreamKind::Gyro,
        6 => Rs2StreamKind::Accel,
        8 => Rs2StreamKind::Pose,
        _ => Rs2StreamKind::Any,
    }
}

fn int_to_format(v: i32) -> Rs2Format {
    match v {
        1 => Rs2Format::Z16,
        3 => Rs2Format::Xyz32f,
        5 => Rs2Format::Rgb8,
        6 => Rs2Format::Bgr8,
        7 => Rs2Format::Rgba8,
        8 => Rs2Format::Bgra8,
        9 => Rs2Format::Y8,
        14 => Rs2Format::Uyvy,
        4 => Rs2Format::Yuyv,
        _ => Rs2Format::Any,
    }
}

/// A list of stream profiles returned by the pipeline or a sensor.
pub struct StreamProfileList {
    pub(crate) handle: *mut ffi::rs2_stream_profile_list,
    pub(crate) owned: bool,
}

impl StreamProfileList {
    /// # Safety
    ///
    /// `handle` must be a valid stream profile list, and ownership is
    /// transferred if the caller previously allocated it with a "should be
    /// released" API.
    pub(crate) unsafe fn from_raw(handle: *mut ffi::rs2_stream_profile_list) -> StreamProfileList {
        StreamProfileList {
            handle,
            owned: true,
        }
    }

    pub fn len(&self) -> usize {
        let mut err = ptr::null_mut();
        let n = unsafe { ffi::rs2_get_stream_profiles_count(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        n.max(0) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> StreamProfile {
        StreamProfile::from_owned_list(self, index as i32)
    }
}

impl Drop for StreamProfileList {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            unsafe { ffi::rs2_delete_stream_profiles_list(self.handle) };
        }
    }
}
