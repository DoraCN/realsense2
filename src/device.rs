use std::ffi::CStr;
use std::ptr;

use crate::error::Rs2Error;
use crate::ffi;
use crate::kind::Rs2CameraInfo;

/// A snapshot of all connected devices at the time of querying.
pub struct DeviceList {
    pub(crate) handle: *mut ffi::rs2_device_list,
}

impl DeviceList {
    pub fn len(&self) -> usize {
        let mut err = ptr::null_mut();
        let n = unsafe { ffi::rs2_get_device_count(self.handle, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return 0;
        }
        n.max(0) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the device at `index`. Each returned device owns an independent
    /// reference and may outlive the list.
    pub fn get(&self, index: usize) -> Option<Device> {
        let mut err = ptr::null_mut();
        let handle = unsafe { ffi::rs2_create_device(self.handle, index as i32, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return None;
        }
        if handle.is_null() {
            return None;
        }
        Some(Device { handle })
    }

    pub fn iter(&self) -> DeviceIter<'_> {
        DeviceIter {
            list: self,
            index: 0,
        }
    }
}

impl Drop for DeviceList {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_device_list(self.handle) };
        }
    }
}

impl<'a> IntoIterator for &'a DeviceList {
    type Item = Device;
    type IntoIter = DeviceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct DeviceIter<'a> {
    list: &'a DeviceList,
    index: usize,
}

impl<'a> Iterator for DeviceIter<'a> {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        let d = self.list.get(self.index)?;
        self.index += 1;
        Some(d)
    }
}

/// A single RealSense device.
///
/// Owns its C handle and releases it via `rs2_delete_device` on drop.
pub struct Device {
    pub(crate) handle: *mut ffi::rs2_device,
}

impl Device {
    /// Adopt an already-created device handle (from device list or pipeline
    /// profile).
    ///
    /// # Safety
    ///
    /// `handle` must be a valid, non-null `rs2_device` that is not used
    /// anywhere else after this call.
    pub(crate) unsafe fn from_raw(handle: *mut ffi::rs2_device) -> Device {
        Device { handle }
    }

    /// Query a device info string.
    pub fn info(&self, info: Rs2CameraInfo) -> Option<String> {
        if !self.supports_info(info) {
            return None;
        }
        let mut err = ptr::null_mut();
        let ptr = unsafe { ffi::rs2_get_device_info(self.handle, info as i32, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return None;
        }
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
    }

    pub fn supports_info(&self, info: Rs2CameraInfo) -> bool {
        let mut err = ptr::null_mut();
        let r = unsafe { ffi::rs2_supports_device_info(self.handle, info as i32, &mut err) };
        if !err.is_null() {
            let _ = unsafe { Rs2Error::from_ptr(err) };
            return false;
        }
        r != 0
    }

    /// Send a hardware reset request to the device.
    pub fn hardware_reset(&self) -> Result<(), Rs2Error> {
        let mut err = ptr::null_mut();
        unsafe { ffi::rs2_hardware_reset(self.handle, &mut err) };
        if !err.is_null() {
            return Err(unsafe { Rs2Error::from_ptr(err) });
        }
        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::rs2_delete_device(self.handle) };
        }
    }
}
