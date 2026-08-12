#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Hand-written FFI bindings to librealsense2 C API.
//!
//! These declarations mirror the headers under `include/librealsense2/h/`
//! from the librealsense repository. Opaque types are represented as
//! zero-sized structs so that pointers are `*mut` to a distinct type.

use std::ffi::c_void;

// ---- Opaque types ---------------------------------------------------------

#[repr(C)]
pub struct rs2_context {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_device {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_device_list {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_pipeline {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_pipeline_profile {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_config {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_frame {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_sensor {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_sensor_list {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_stream_profile {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_stream_profile_list {
    _private: [u8; 0],
}
#[repr(C)]
pub struct rs2_error {
    _private: [u8; 0],
}

// ---- Enums (C ints) -------------------------------------------------------

pub type rs2_camera_info = i32;
pub type rs2_stream = i32;
pub type rs2_format = i32;
pub type rs2_distortion = i32;
pub type rs2_extension = i32;
pub type rs2_time_t = f64;
pub type rs2_metadata_type = i64;

// ---- Value structs --------------------------------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct rs2_intrinsics {
    pub width: i32,
    pub height: i32,
    pub ppx: f32,
    pub ppy: f32,
    pub fx: f32,
    pub fy: f32,
    pub model: rs2_distortion,
    pub coeffs: [f32; 5],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct rs2_extrinsics {
    pub rotation: [f32; 9],
    pub translation: [f32; 3],
}

// ---- Error handling -------------------------------------------------------

extern "C" {
    pub fn rs2_get_failed_function(error: *const rs2_error) -> *const c_char;
    pub fn rs2_get_failed_args(error: *const rs2_error) -> *const c_char;
    pub fn rs2_get_error_message(error: *const rs2_error) -> *const c_char;
    pub fn rs2_free_error(error: *mut rs2_error);

    pub fn rs2_create_error(
        what: *const c_char,
        name: *const c_char,
        args: *const c_char,
        type_: i32,
    ) -> *mut rs2_error;
}

use std::ffi::c_char;

// ---- Context --------------------------------------------------------------

extern "C" {
    pub fn rs2_create_context(api_version: i32, error: *mut *mut rs2_error) -> *mut rs2_context;
    pub fn rs2_create_context_ex(
        api_version: i32,
        json_settings: *const c_char,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_context;
    pub fn rs2_delete_context(context: *mut rs2_context);
    pub fn rs2_query_devices(context: *const rs2_context, error: *mut *mut rs2_error)
        -> *mut rs2_device_list;
    pub fn rs2_query_devices_ex(
        context: *const rs2_context,
        product_mask: i32,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_device_list;
}

// ---- Device list / device -------------------------------------------------

extern "C" {
    pub fn rs2_get_device_count(list: *const rs2_device_list, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_delete_device_list(list: *mut rs2_device_list);
    pub fn rs2_create_device(
        list: *const rs2_device_list,
        index: i32,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_device;
    pub fn rs2_delete_device(device: *mut rs2_device);
    pub fn rs2_get_device_info(
        device: *const rs2_device,
        info: rs2_camera_info,
        error: *mut *mut rs2_error,
    ) -> *const c_char;
    pub fn rs2_supports_device_info(
        device: *const rs2_device,
        info: rs2_camera_info,
        error: *mut *mut rs2_error,
    ) -> i32;
    pub fn rs2_hardware_reset(device: *const rs2_device, error: *mut *mut rs2_error);
    pub fn rs2_query_sensors(
        device: *const rs2_device,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_sensor_list;
}

// ---- Config ---------------------------------------------------------------

extern "C" {
    pub fn rs2_create_config(error: *mut *mut rs2_error) -> *mut rs2_config;
    pub fn rs2_delete_config(config: *mut rs2_config);
    pub fn rs2_config_enable_stream(
        config: *mut rs2_config,
        stream: rs2_stream,
        index: i32,
        width: i32,
        height: i32,
        format: rs2_format,
        framerate: i32,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_config_enable_all_stream(config: *mut rs2_config, error: *mut *mut rs2_error);
    pub fn rs2_config_enable_device(
        config: *mut rs2_config,
        serial: *const c_char,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_config_disable_all_streams(config: *mut rs2_config, error: *mut *mut rs2_error);
    pub fn rs2_config_disable_stream(
        config: *mut rs2_config,
        stream: rs2_stream,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_config_disable_indexed_stream(
        config: *mut rs2_config,
        stream: rs2_stream,
        index: i32,
        error: *mut *mut rs2_error,
    );
}

// ---- Pipeline -------------------------------------------------------------

extern "C" {
    pub fn rs2_create_pipeline(context: *mut rs2_context, error: *mut *mut rs2_error)
        -> *mut rs2_pipeline;
    pub fn rs2_delete_pipeline(pipe: *mut rs2_pipeline);
    pub fn rs2_pipeline_stop(pipe: *mut rs2_pipeline, error: *mut *mut rs2_error);
    pub fn rs2_pipeline_start(
        pipe: *mut rs2_pipeline,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_pipeline_profile;
    pub fn rs2_pipeline_start_with_config(
        pipe: *mut rs2_pipeline,
        config: *mut rs2_config,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_pipeline_profile;
    pub fn rs2_pipeline_wait_for_frames(
        pipe: *mut rs2_pipeline,
        timeout_ms: u32,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_frame;
    pub fn rs2_pipeline_poll_for_frames(
        pipe: *mut rs2_pipeline,
        output_frame: *mut *mut rs2_frame,
        error: *mut *mut rs2_error,
    ) -> i32;
    pub fn rs2_pipeline_try_wait_for_frames(
        pipe: *mut rs2_pipeline,
        output_frame: *mut *mut rs2_frame,
        timeout_ms: u32,
        error: *mut *mut rs2_error,
    ) -> i32;
    pub fn rs2_pipeline_get_active_profile(
        pipe: *mut rs2_pipeline,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_pipeline_profile;
    pub fn rs2_delete_pipeline_profile(profile: *mut rs2_pipeline_profile);
    pub fn rs2_pipeline_profile_get_device(
        profile: *mut rs2_pipeline_profile,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_device;
    pub fn rs2_pipeline_profile_get_streams(
        profile: *mut rs2_pipeline_profile,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_stream_profile_list;
}

// ---- Stream profiles ------------------------------------------------------

extern "C" {
    pub fn rs2_get_stream_profile_data(
        mode: *const rs2_stream_profile,
        stream: *mut rs2_stream,
        format: *mut rs2_format,
        index: *mut i32,
        unique_id: *mut i32,
        framerate: *mut i32,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_get_video_stream_resolution(
        mode: *const rs2_stream_profile,
        width: *mut i32,
        height: *mut i32,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_get_video_stream_intrinsics(
        mode: *const rs2_stream_profile,
        intrinsics: *mut rs2_intrinsics,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_get_extrinsics(
        from: *const rs2_stream_profile,
        to: *const rs2_stream_profile,
        extrin: *mut rs2_extrinsics,
        error: *mut *mut rs2_error,
    );
    pub fn rs2_get_stream_profiles_count(
        list: *const rs2_stream_profile_list,
        error: *mut *mut rs2_error,
    ) -> i32;
    pub fn rs2_get_stream_profile(
        list: *const rs2_stream_profile_list,
        index: i32,
        error: *mut *mut rs2_error,
    ) -> *const rs2_stream_profile;
    pub fn rs2_delete_stream_profiles_list(list: *mut rs2_stream_profile_list);
}

// ---- Frames ---------------------------------------------------------------

extern "C" {
    pub fn rs2_release_frame(frame: *mut rs2_frame);
    pub fn rs2_frame_add_ref(frame: *mut rs2_frame, error: *mut *mut rs2_error);
    pub fn rs2_get_frame_data(frame: *const rs2_frame, error: *mut *mut rs2_error)
        -> *const c_void;
    pub fn rs2_get_frame_data_size(frame: *const rs2_frame, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_get_frame_width(frame: *const rs2_frame, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_get_frame_height(frame: *const rs2_frame, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_get_frame_stride_in_bytes(frame: *const rs2_frame, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_get_frame_bits_per_pixel(frame: *const rs2_frame, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_get_frame_timestamp(frame: *const rs2_frame, error: *mut *mut rs2_error)
        -> rs2_time_t;
    pub fn rs2_get_frame_number(frame: *const rs2_frame, error: *mut *mut rs2_error) -> u64;
    pub fn rs2_get_frame_stream_profile(
        frame: *const rs2_frame,
        error: *mut *mut rs2_error,
    ) -> *const rs2_stream_profile;
    pub fn rs2_is_frame_extendable_to(
        frame: *const rs2_frame,
        extension_type: rs2_extension,
        error: *mut *mut rs2_error,
    ) -> i32;
    pub fn rs2_depth_frame_get_units(frame: *const rs2_frame, error: *mut *mut rs2_error) -> f32;
    pub fn rs2_extract_frame(
        composite: *mut rs2_frame,
        index: i32,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_frame;
    pub fn rs2_embedded_frames_count(composite: *mut rs2_frame, error: *mut *mut rs2_error) -> i32;
    pub fn rs2_keep_frame(frame: *mut rs2_frame);
}

// ---- Sensors --------------------------------------------------------------

extern "C" {
    pub fn rs2_delete_sensor_list(list: *mut rs2_sensor_list);
    pub fn rs2_get_sensors_count(
        list: *const rs2_sensor_list,
        error: *mut *mut rs2_error,
    ) -> i32;
    pub fn rs2_create_sensor(
        list: *const rs2_sensor_list,
        index: i32,
        error: *mut *mut rs2_error,
    ) -> *mut rs2_sensor;
    pub fn rs2_delete_sensor(sensor: *mut rs2_sensor);
    pub fn rs2_get_depth_scale(sensor: *mut rs2_sensor, error: *mut *mut rs2_error) -> f32;
}

// ---- Misc helper functions ------------------------------------------------

extern "C" {
    pub fn rs2_camera_info_to_string(info: rs2_camera_info) -> *const c_char;
    pub fn rs2_stream_to_string(stream: rs2_stream) -> *const c_char;
    pub fn rs2_format_to_string(format: rs2_format) -> *const c_char;
    pub fn rs2_distortion_to_string(distortion: rs2_distortion) -> *const c_char;
    pub fn rs2_get_api_version(error: *mut *mut rs2_error) -> i32;
}
