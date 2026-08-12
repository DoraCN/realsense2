use std::ptr;

use crate::error::Rs2Error;
use crate::ffi;
use crate::device::DeviceList;

/// API version constant passed to `rs2_create_context`.
pub const API_VERSION: i32 = 2 * 10000 + 56 * 100 + 5; // 2.56.5

/// Top-level entry point into the RealSense API.
///
/// A context tracks all connected devices. Create it before querying devices
/// or building pipelines.
pub struct Context {
    pub(crate) handle: *mut ffi::rs2_context,
}

impl Context {
    /// Create a new context.
    pub fn new() -> Result<Self, Rs2Error> {
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_create_context(API_VERSION, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(Context { handle })
    }

    /// Create a new context with advanced JSON settings (see
    /// `rs2_create_context_ex` in the SDK documentation).
    pub fn new_with_settings(settings: &str) -> Result<Self, Rs2Error> {
        let c_settings = std::ffi::CString::new(settings).map_err(|_| Rs2Error {
            message: "settings string contains NUL byte".into(),
            function: "Context::new_with_settings".into(),
            args: "".into(),
        })?;
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_create_context_ex(API_VERSION, c_settings.as_ptr(), &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(Context { handle })
    }

    /// Query all RealSense-compatible devices connected to the host.
    pub fn query_devices(&self) -> Result<DeviceList, Rs2Error> {
        let mut err = ptr::null_mut();
        let list = unsafe { ffi::rs2_query_devices(self.handle, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(DeviceList { handle: list })
    }

    /// Query devices filtered by a product-line bitmask (see `kind::product_line`).
    pub fn query_devices_ex(&self, product_mask: i32) -> Result<DeviceList, Rs2Error> {
        let mut err = ptr::null_mut();
        let list = unsafe { ffi::rs2_query_devices_ex(self.handle, product_mask, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(DeviceList { handle: list })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_context(self.handle) };
        }
    }
}

/// Query the SDK API version.
pub fn api_version() -> i32 {
    let mut err = ptr::null_mut();
    let v = unsafe { ffi::rs2_get_api_version(&mut err) };
    if !err.is_null() {
        let _ = unsafe { Rs2Error::from_ptr(err) };
        return 0;
    }
    v
}
