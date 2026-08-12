use std::error::Error;
use std::ffi::{CStr, c_char};
use std::fmt;

use crate::ffi;
use crate::kind::Rs2ExceptionType;

/// Error returned by librealsense2 through the `rs2_error` out-parameter.
#[derive(Debug)]
pub struct Rs2Error {
    pub message: String,
    pub function: String,
    pub args: String,
}

impl Rs2Error {
    /// Take ownership of a C error and convert it into a Rust error.
    ///
    /// # Safety
    ///
    /// `error` must be a valid pointer returned by the library that has not
    /// yet been freed.
    pub(crate) unsafe fn from_ptr(error: *mut ffi::rs2_error) -> Self {
        let msg = c_str_or(ffi::rs2_get_error_message(error), "<no message>");
        let func = c_str_or(ffi::rs2_get_failed_function(error), "<unknown function>");
        let args = c_str_or(ffi::rs2_get_failed_args(error), "");
        ffi::rs2_free_error(error);
        Rs2Error {
            message: msg,
            function: func,
            args,
        }
    }
}

unsafe fn c_str_or(ptr: *const c_char, default: &str) -> String {
    if ptr.is_null() {
        default.to_string()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

impl fmt::Display for Rs2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "realsense2[{}]: {}", self.function, self.message)
    }
}

impl Error for Rs2Error {}

/// Raw librealsense2 exception type, propagated for callers that want it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rs2Exception {
    pub kind: Rs2ExceptionType,
}

impl fmt::Display for Rs2Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.kind.to_str().unwrap_or("unknown exception type")
        )
    }
}

impl Error for Rs2Exception {}
