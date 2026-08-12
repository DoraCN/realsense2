//! Post-processing blocks: depth filters, alignment, and point-cloud
//! generation.
//!
//! All blocks are implemented on top of librealsense's *processing block*
//! mechanism, fed through an internal frame queue (`rs2_start_processing_queue`).
//! This avoids Rust-side callbacks and keeps the design simple and safe.

use std::ffi::CString;
use std::ptr;

use crate::error::Rs2Error;
use crate::ffi;
use crate::frame::{Frame, FrameSet};
use crate::kind::{Rs2Extension, Rs2Option, Rs2StreamKind};

/// A frame queue feeding a processing block. Frames enqueued to the block come
/// out the other end (possibly transformed) and can be dequeued by the caller.
struct BlockQueue {
    handle: *mut ffi::rs2_frame_queue,
}

impl BlockQueue {
    fn new(capacity: i32) -> Result<BlockQueue, Rs2Error> {
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_create_frame_queue(capacity, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(BlockQueue { handle })
    }

    /// Block up to `timeout_ms` for the next processed frame.
    fn wait_for_frame(&self, timeout_ms: u32) -> Result<Option<Frame>, Rs2Error> {
        let mut err = ptr::null_mut();
        let frame = unsafe { ffi::rs2_wait_for_frame(self.handle, timeout_ms, &mut err) };
        if !err.is_null() {
            let e = unsafe { Rs2Error::from_ptr(err) };
            // "Frame did not arrive in time!" / "didn't arrive" -> timeout,
            // not a hard error.
            if e.message.contains("didn't arrive") || e.message.contains("did not arrive") {
                return Ok(None);
            }
            return Err(e);
        }
        if frame.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { Frame::from_raw(frame) }))
    }
}

impl Drop for BlockQueue {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_frame_queue(self.handle) };
        }
    }
}

/// A processing block that transforms input frames. Holds the block handle and
/// its output queue.
pub struct ProcessingBlock {
    handle: *mut ffi::rs2_processing_block,
    queue: BlockQueue,
}

impl ProcessingBlock {
    fn new(
        create: unsafe extern "C" fn(*mut *mut ffi::rs2_error) -> *mut ffi::rs2_processing_block,
    ) -> Result<ProcessingBlock, Rs2Error> {
        let mut err = ptr::null_mut();
        let handle = unsafe { create(&mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        let queue = BlockQueue::new(4)?;
        let mut start_err = ptr::null_mut();
        unsafe { ffi::rs2_start_processing_queue(handle, queue.handle, &mut start_err) };
        if !start_err.is_null() {
            unsafe { ffi::rs2_delete_processing_block(handle) };
            return Err(unsafe { Rs2Error::from_ptr(start_err) });
        }
        Ok(ProcessingBlock { handle, queue })
    }

    /// Send a single frame into the block. Ownership is moved to the block.
    pub fn process_frame(&self, frame: &Frame) -> Result<(), Rs2Error> {
        // rs2_process_frame takes ownership; hand it a fresh reference.
        let mut add_err = ptr::null_mut();
        unsafe { ffi::rs2_frame_add_ref(frame.raw(), &mut add_err) };
        if !add_err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(add_err) });
        }
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_process_frame(self.handle, frame.raw(), &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(())
    }

    /// Send an entire frameset (all streams) into the block. Used by alignment
    /// and point-cloud blocks, which need the color frame alongside depth.
    pub fn process_frameset(&self, frameset: &FrameSet) -> Result<(), Rs2Error> {
        let mut add_err = ptr::null_mut();
        unsafe { ffi::rs2_frame_add_ref(frameset.raw(), &mut add_err) };
        if !add_err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(add_err) });
        }
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_process_frame(self.handle, frameset.raw(), &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(())
    }

    /// Get the next processed frame from the output queue.
    pub fn output(&self, timeout_ms: u32) -> Result<Option<Frame>, Rs2Error> {
        self.queue.wait_for_frame(timeout_ms)
    }

    /// Set an option (e.g. filter parameters) on this block.
    pub fn set_option(&self, option: Rs2Option, value: f32) -> Result<(), Rs2Error> {
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_set_option(self.handle as *const _, option as i32, value, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(())
    }

    pub fn get_option(&self, option: Rs2Option) -> Result<f32, Rs2Error> {
        let mut err = ptr::null_mut();
        let v = unsafe { ffi::rs2_get_option(self.handle as *const _, option as i32, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(v)
    }

    /// Wait for the next *Points* frame from the output queue.
    ///
    /// Some blocks (point cloud, alignment) emit multiple frames per input
    /// frameset — e.g. the point-cloud block outputs the generated Points frame
    /// plus passes through the color frame untouched. This method skips
    /// non-Points frames until a Points frame arrives or the timeout elapses.
    pub fn output_points(&self, timeout_ms: u32) -> Result<Option<Frame>, Rs2Error> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            let remaining = deadline
                .saturating_duration_since(std::time::Instant::now())
                .as_millis();
            if remaining == 0 {
                return Ok(None);
            }
            let Some(frame) = self.output(remaining as u32)? else {
                return Ok(None);
            };
            if frame.is_extendable_to(Rs2Extension::Points) {
                return Ok(Some(frame));
            }
            // non-Points frame (e.g. passthrough color): drop and keep waiting
        }
    }
}

impl Drop for ProcessingBlock {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_processing_block(self.handle) };
        }
    }
}

/// Depth decimation filter: downsamples the depth image, magnifying remaining
/// pixels (option `FilterMagnitude`, 2-8, default 2).
pub fn decimation() -> Result<ProcessingBlock, Rs2Error> {
    ProcessingBlock::new(ffi::rs2_create_decimation_filter_block)
}

/// Spatial filter: smooths depth while preserving edges (options
/// `FilterMagnitude`, `FilterSmoothAlpha`, `FilterSmoothDelta`).
pub fn spatial() -> Result<ProcessingBlock, Rs2Error> {
    ProcessingBlock::new(ffi::rs2_create_spatial_filter_block)
}

/// Temporal filter: removes noise by blending across frames (options
/// `FilterSmoothAlpha`, `FilterSmoothDelta`, `HolesFill`).
pub fn temporal() -> Result<ProcessingBlock, Rs2Error> {
    ProcessingBlock::new(ffi::rs2_create_temporal_filter_block)
}

/// Hole filling filter: fills invalid depth pixels from neighbors (option
/// `HolesFill`, 0-2).
pub fn hole_filling() -> Result<ProcessingBlock, Rs2Error> {
    ProcessingBlock::new(ffi::rs2_create_hole_filling_filter_block)
}

/// Depth-to-color alignment block. Aligns the depth stream to the color
/// stream's viewpoint.
pub fn align_to_color() -> Result<ProcessingBlock, Rs2Error> {
    let mut err = ptr::null_mut();
    let handle = unsafe { ffi::rs2_create_align(Rs2StreamKind::Color as i32, &mut err) };
    if !err.is_null() {
        return Err(unsafe { Rs2Error::from_ptr(err) });
    }
    unsafe { ProcessingBlock::from_raw(handle) }
}

/// Point-cloud generation block. Generates a `Points` frame from depth (and
/// optionally texture from a color frame).
pub fn pointcloud() -> Result<ProcessingBlock, Rs2Error> {
    ProcessingBlock::new(ffi::rs2_create_pointcloud)
}

impl ProcessingBlock {
    /// # Safety
    ///
    /// `handle` must be a valid processing block whose ownership is
    /// transferred to the caller.
    unsafe fn from_raw(handle: *mut ffi::rs2_processing_block) -> Result<ProcessingBlock, Rs2Error> {
        let queue = BlockQueue::new(4)?;
        let mut start_err = ptr::null_mut();
        unsafe { ffi::rs2_start_processing_queue(handle, queue.handle, &mut start_err) };
        if !start_err.is_null() {
            unsafe { ffi::rs2_delete_processing_block(handle) };
            return Err(unsafe { Rs2Error::from_ptr(start_err) });
        }
        Ok(ProcessingBlock { handle, queue })
    }
}

/// Export a Points frame to a PLY file. `texture` optionally provides the color
/// texture frame.
pub fn export_ply(frame: &Frame, path: &str, texture: Option<&Frame>) -> Result<(), Rs2Error> {
    let c_path = CString::new(path).map_err(|_| Rs2Error {
        message: "path contains NUL byte".into(),
        function: "export_ply".into(),
        args: "".into(),
    })?;
    let mut err = ptr::null_mut();
    unsafe {
        ffi::rs2_export_to_ply(
            frame.raw(),
            c_path.as_ptr(),
            texture.map_or(ptr::null_mut(), |f| f.raw()),
            &mut err,
        )
    };
    if !err.is_null() {
        return Err(unsafe { Rs2Error::from_ptr(err) });
    }
    Ok(())
}
