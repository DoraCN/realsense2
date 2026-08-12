#![allow(non_camel_case_types)]

use std::ffi::CStr;

use crate::ffi;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs2ExceptionType {
    Unknown = 0,
    CameraDisconnected = 1,
    Backend = 2,
    InvalidValue = 3,
    WrongApiCallSequence = 4,
    NotImplemented = 5,
    DeviceInRecoveryMode = 6,
    Io = 7,
    Count = 8,
}

impl Rs2ExceptionType {
    pub fn to_str(self) -> Option<&'static str> {
        Some(match self {
            Rs2ExceptionType::Unknown => "unknown",
            Rs2ExceptionType::CameraDisconnected => "camera disconnected",
            Rs2ExceptionType::Backend => "backend",
            Rs2ExceptionType::InvalidValue => "invalid value",
            Rs2ExceptionType::WrongApiCallSequence => "wrong api call sequence",
            Rs2ExceptionType::NotImplemented => "not implemented",
            Rs2ExceptionType::DeviceInRecoveryMode => "device in recovery mode",
            Rs2ExceptionType::Io => "io",
            Rs2ExceptionType::Count => return None,
        })
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs2CameraInfo {
    Name = 0,
    SerialNumber = 1,
    FirmwareVersion = 2,
    RecommendedFirmwareVersion = 3,
    PhysicalPort = 4,
    DebugOpCode = 5,
    AdvancedMode = 6,
    ProductId = 7,
    CameraLocked = 8,
    UsbTypeDescriptor = 9,
    ProductLine = 10,
    AsicSerialNumber = 11,
    FirmwareUpdateId = 12,
    IpAddress = 13,
    DfuDevicePath = 14,
    ConnectionType = 15,
    SmcuFwVersion = 16,
    ImuType = 17,
    MipiDriverVersion = 18,
    Count = 19,
}

impl Rs2CameraInfo {
    pub fn to_str(self) -> Option<&'static str> {
        let p = self as ffi::rs2_camera_info;
        let ptr = unsafe { ffi::rs2_camera_info_to_string(p) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("<invalid>"))
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs2StreamKind {
    Any = 0,
    Depth = 1,
    Color = 2,
    Infrared = 3,
    Fisheye = 4,
    Gyro = 5,
    Accel = 6,
    Gpio = 7,
    Pose = 8,
    Confidence = 9,
    Motion = 10,
    Safety = 11,
    Occupancy = 12,
    LabeledPointCloud = 13,
    ObjectDetection = 14,
    Count = 15,
}

impl Rs2StreamKind {
    pub fn to_str(self) -> Option<&'static str> {
        let p = self as ffi::rs2_stream;
        let ptr = unsafe { ffi::rs2_stream_to_string(p) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("<invalid>"))
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs2Format {
    Any = 0,
    Z16 = 1,
    Disparity16 = 2,
    Xyz32f = 3,
    Yuyv = 4,
    Rgb8 = 5,
    Bgr8 = 6,
    Rgba8 = 7,
    Bgra8 = 8,
    Y8 = 9,
    Y16 = 10,
    Raw10 = 11,
    Raw16 = 12,
    Raw8 = 13,
    Uyvy = 14,
    MotionRaw = 15,
    MotionXyz32f = 16,
    GpioRaw = 17,
    SixDof = 18,
    Disparity32 = 19,
    Y10bpack = 20,
    Distance = 21,
    Mjpeg = 22,
    Y8i = 23,
    Y12i = 24,
    Inzi = 25,
    Invi = 26,
    W10 = 27,
    Z16h = 28,
    Fg = 29,
    Y411 = 30,
    Y16i = 31,
    M420 = 32,
    CombinedMotion = 33,
    Nv12 = 34,
    Count = 35,
}

impl Rs2Format {
    pub fn to_str(self) -> Option<&'static str> {
        let p = self as ffi::rs2_format;
        let ptr = unsafe { ffi::rs2_format_to_string(p) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("<invalid>"))
        }
    }
}

/// Bitmask flags for `Context::query_devices_ex` / product line selection.
pub mod product_line {
    pub const ANY: i32 = 0xff;
    pub const ANY_INTEL: i32 = 0xfe;
    pub const NON_INTEL: i32 = 0x01;
    pub const D400: i32 = 0x02;
    pub const SR300: i32 = 0x04;
    pub const L500: i32 = 0x08;
    pub const T200: i32 = 0x10;
    pub const D500: i32 = 0x20;
    pub const SW_ONLY: i32 = 0x100;
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs2Distortion {
    None = 0,
    ModifiedBrownConrady = 1,
    InverseBrownConrady = 2,
    Ftheta = 3,
    BrownConrady = 4,
    KannalaBrandt4 = 5,
    Count = 6,
}

impl Rs2Distortion {
    pub fn to_str(self) -> Option<&'static str> {
        let p = self as ffi::rs2_distortion;
        let ptr = unsafe { ffi::rs2_distortion_to_string(p) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("<invalid>"))
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs2Extension {
    Unknown = 0,
    Debug = 1,
    Info = 2,
    Motion = 3,
    Options = 4,
    Video = 5,
    Roi = 6,
    DepthSensor = 7,
    VideoFrame = 8,
    MotionFrame = 9,
    CompositeFrame = 10,
    Points = 11,
    DepthFrame = 12,
    AdvancedMode = 13,
    Record = 14,
    VideoProfile = 15,
    Playback = 16,
    DepthStereoSensor = 17,
    DisparityFrame = 18,
    MotionProfile = 19,
    PoseFrame = 20,
    PoseProfile = 21,
    Tm2 = 22,
    SoftwareDevice = 23,
    SoftwareSensor = 24,
    DecimationFilter = 25,
    ThresholdFilter = 26,
    DisparityFilter = 27,
    SpatialFilter = 28,
    TemporalFilter = 29,
    HoleFillingFilter = 30,
    ZeroOrderFilter = 31,
    RecommendedFilters = 32,
    Pose = 33,
    PoseSensor = 34,
    WheelOdometer = 35,
    GlobalTimer = 36,
    Updatable = 37,
    UpdateDevice = 38,
    L500DepthSensor = 39,
    Tm2Sensor = 40,
    AutoCalibratedDevice = 41,
    ColorSensor = 42,
    MotionSensor = 43,
    FisheyeSensor = 44,
    DepthHuffmanDecoder = 45,
    Serializable = 46,
    FwLogger = 47,
    AutoCalibrationFilter = 48,
    DeviceCalibration = 49,
    CalibratedSensor = 50,
    HdrMerge = 51,
    SequenceIdFilter = 52,
    MaxUsableRangeSensor = 53,
    DebugStreamSensor = 54,
    CalibrationChangeDevice = 55,
    RotationFilter = 56,
    SafetySensor = 57,
    DepthMappingSensor = 58,
    LabeledPoints = 59,
    EthConfig = 60,
    SupportedEmbeddedFilters = 61,
    DecimationEmbeddedFilter = 62,
    TemporalEmbeddedFilter = 63,
    CloseRangeEmbeddedFilter = 64,
    InferenceFrame = 65,
    ObjectDetectionFrame = 66,
    InferenceSensor = 67,
    ObjectDetectionSensor = 68,
    InferenceProfile = 69,
    Count = 70,
}
