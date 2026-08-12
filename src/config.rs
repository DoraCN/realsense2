use std::ffi::CString;
use std::ptr;

use crate::error::Rs2Error;
use crate::ffi;
use crate::kind::{Rs2Format, Rs2StreamKind};

/// Builds stream configuration requests for a [`crate::Pipeline`].
///
/// Mirrors the C `rs2_config` object. A config is a set of filters; once
/// passed to `Pipeline::start`, its contents are resolved against connected
/// devices.
pub struct Config {
    pub(crate) handle: *mut ffi::rs2_config,
}

impl Config {
    pub fn new() -> Result<Config, Rs2Error> {
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_create_config(&mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(Config { handle })
    }

    /// Enable a stream with the given parameters. Use `0` for any width,
    /// height, framerate, and `None` for any index.
    pub fn enable_stream(
        &mut self,
        stream: Rs2StreamKind,
        index: Option<i32>,
        width: i32,
        height: i32,
        format: Rs2Format,
        framerate: i32,
    ) -> Result<&mut Self, Rs2Error> {
        let mut err = ptr::null_mut();
        unsafe {
            ffi::rs2_config_enable_stream(
                self.handle,
                stream as i32,
                index.unwrap_or(-1),
                width,
                height,
                format as i32,
                framerate,
                &mut err,
            )
        };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(self)
    }

    /// Enable all raw streams of the selected device.
    pub fn enable_all_streams(&mut self) -> Result<&mut Self, Rs2Error> {
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_config_enable_all_stream(self.handle, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(self)
    }

    /// Select a specific device by serial number.
    pub fn enable_device(&mut self, serial: &str) -> Result<&mut Self, Rs2Error> {
        let c_serial = CString::new(serial).map_err(|_| Rs2Error {
            message: "serial contains NUL byte".into(),
            function: "Config::enable_device".into(),
            args: "".into(),
        })?;
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_config_enable_device(self.handle, c_serial.as_ptr(), &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(self)
    }

    /// Disable all streams previously enabled.
    pub fn disable_all_streams(&mut self) -> Result<&mut Self, Rs2Error> {
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_config_disable_all_streams(self.handle, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(self)
    }

    /// Disable a single stream type.
    pub fn disable_stream(&mut self, stream: Rs2StreamKind) -> Result<&mut Self, Rs2Error> {
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_config_disable_stream(self.handle, stream as i32, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(self)
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_config(self.handle) };
        }
    }
}
