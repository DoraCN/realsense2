use std::ptr;

use crate::config::Config;
use crate::device::Device;
use crate::error::Rs2Error;
use crate::ffi;
use crate::frame::FrameSet;
use crate::stream_profile::StreamProfileList;

/// Controls device streaming. Holds a C `rs2_pipeline` handle.
///
/// A pipeline is created from a [`crate::Context`], configured with a
/// [`Config`], and started; then frames are retrieved with `wait_for_frames`
/// or polled with `poll_for_frames`.
pub struct Pipeline {
    handle: *mut ffi::rs2_pipeline,
    started: bool,
}

impl Pipeline {
    /// Create a new pipeline bound to the given context.
    pub fn new(context: &crate::Context) -> Result<Pipeline, Rs2Error> {
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_create_pipeline(context.handle, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(Pipeline {
            handle,
            started: false,
        })
    }

    /// Start streaming with the default configuration.
    pub fn start(&mut self) -> Result<(), Rs2Error> {
        self.start_with_config(None)
    }

    /// Start streaming according to the given configuration.
    pub fn start_with_config(&mut self, config: Option<&Config>) -> Result<(), Rs2Error> {
        let mut err = ptr::null_mut();
        let profile_ptr = unsafe {
            match config {
                Some(c) => ffi::rs2_pipeline_start_with_config(self.handle, c.handle, &mut err),
                None => ffi::rs2_pipeline_start(self.handle, &mut err),
            }
        };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        if !profile_ptr.is_null() {
            unsafe { ffi::rs2_delete_pipeline_profile(profile_ptr) };
        }
        self.started = true;
        Ok(())
    }

    /// Block until a new set of frames is available, or until `timeout_ms`
    /// elapses. Returns `None` on timeout.
    pub fn wait_for_frames(&mut self, timeout_ms: u32) -> Result<FrameSet, Rs2Error> {
        let mut err = ptr::null_mut();
        let frame_ptr =
            unsafe { ffi::rs2_pipeline_wait_for_frames(self.handle, timeout_ms, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        if frame_ptr.is_null() {
            return Err(Rs2Error {
                message: "no frame received before timeout".into(),
                function: "rs2_pipeline_wait_for_frames".into(),
                args: "".into(),
            });
        }
        Ok(unsafe { FrameSet::from_raw(frame_ptr) })
    }

    /// Non-blocking frame retrieval. Returns `None` if no new frameset is
    /// available.
    pub fn poll_for_frames(&mut self) -> Result<Option<FrameSet>, Rs2Error> {
        let mut err = ptr::null_mut();
        let mut frame_ptr = ptr::null_mut();
        let available =
            unsafe { ffi::rs2_pipeline_poll_for_frames(self.handle, &mut frame_ptr, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        if available == 0 || frame_ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { FrameSet::from_raw(frame_ptr) }))
    }

    /// Retrieve the active pipeline profile (device + streams) after start.
    pub fn get_active_profile(&self) -> Result<ActiveProfile, Rs2Error> {
        let mut err = ptr::null_mut();
        let profile_ptr = unsafe { ffi::rs2_pipeline_get_active_profile(self.handle, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        if profile_ptr.is_null() {
            return Err(Rs2Error {
                message: "pipeline is not active".into(),
                function: "rs2_pipeline_get_active_profile".into(),
                args: "".into(),
            });
        }
        Ok(ActiveProfile {
            handle: profile_ptr,
        })
    }

    /// Stop streaming. The pipeline is stopped automatically on drop.
    pub fn stop(&mut self) -> Result<(), Rs2Error> {
        if self.started {
            let mut err = ptr::null_mut();
            unsafe { ffi::rs2_pipeline_stop(self.handle, &mut err) };
            if !err.is_null() {
                return Err(unsafe { Rs2Error::from_ptr(err) });
            }
            self.started = false;
        }
        Ok(())
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        // rs2_delete_pipeline stops the pipeline implicitly.
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_pipeline(self.handle) };
        }
    }
}

/// The resolved device and stream profiles of an active pipeline.
pub struct ActiveProfile {
    pub(crate) handle: *mut ffi::rs2_pipeline_profile,
}

impl ActiveProfile {
    /// The device used by the pipeline.
    ///
    /// The C API returns a reference-counted handle that must be released with
    /// `rs2_delete_device` (the C++ wrapper does exactly this), so ownership
    /// is transferred to the returned [`Device`].
    pub fn device(&self) -> Device {
        let mut err = ptr::null_mut();
        let device_ptr = unsafe { ffi::rs2_pipeline_profile_get_device(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
        }
        unsafe { Device::from_raw(device_ptr) }
    }

    /// The stream profiles active in the pipeline.
    pub fn streams(&self) -> StreamProfileList {
        let mut err = ptr::null_mut();
        let list_ptr =
            unsafe { ffi::rs2_pipeline_profile_get_streams(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
        }
        unsafe { StreamProfileList::from_raw(list_ptr) }
    }
}

impl Drop for ActiveProfile {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_pipeline_profile(self.handle) };
        }
    }
}
