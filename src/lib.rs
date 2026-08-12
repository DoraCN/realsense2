//! Hand-written FFI bindings and safe wrappers for the Intel RealSense SDK 2.0
//! (librealsense2).
//!
//! This crate does **not** depend on the third-party `realsense-rust` crate.
//! Instead it declares the necessary `extern "C"` bindings by hand (see the
//! [`ffi`] module) and exposes safe, RAII-based wrappers on top.
//!
//! # Requirements
//!
//! librealsense2 must be installed on the system. The build script locates it
//! via `pkg-config realsense2` or by searching standard library paths.
//!
//! # Example
//!
//! ```
//! use realsense2::{Context, Rs2StreamKind, Rs2Format};
//!
//! let context = Context::new()?;
//! let devices = context.query_devices()?;
//! assert!(!devices.is_empty(), "no RealSense device found");
//! # Ok::<(), realsense2::error::Rs2Error>(())
//! ```

#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod config;
pub mod context;
pub mod device;
pub mod error;
pub mod ffi;
pub mod frame;
pub mod kind;
pub mod pipeline;
pub mod processing;
pub mod stream_profile;

pub use config::Config;
pub use context::{api_version, Context};
pub use device::{Device, DeviceList};
pub use error::Rs2Error;
pub use frame::{Frame, FrameSet};
pub use kind::{
    Rs2CameraInfo, Rs2Distortion, Rs2Extension, Rs2Format, Rs2Option, Rs2StreamKind,
};
pub use pipeline::{ActiveProfile, Pipeline};
pub use processing::{align_to_color, decimation, export_ply, hole_filling, pointcloud, spatial, temporal, ProcessingBlock};
pub use stream_profile::{StreamProfile, StreamProfileList};
