//! Error type mapping SDK return codes to Rust.

use std::os::raw::c_int;
use svbony_sys::*;

/// Errors returned by the SVBony camera SDK.
///
/// Each variant corresponds to a `SVB_ERROR_*` code from the C SDK.
/// [`Unknown`](Error::Unknown) wraps any code not recognised by this crate
/// (e.g. from a newer SDK version).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// No camera connected or index value out of range.
    #[error("invalid camera index")]
    InvalidIndex,
    /// No camera with this ID is connected.
    #[error("invalid camera ID")]
    InvalidId,
    /// Invalid control type passed to get/set.
    #[error("invalid control type")]
    InvalidControlType,
    /// The camera has not been opened.
    #[error("camera not open")]
    CameraClosed,
    /// Camera was physically disconnected.
    #[error("camera removed")]
    CameraRemoved,
    /// File path does not exist.
    #[error("invalid path")]
    InvalidPath,
    /// Unsupported file format.
    #[error("invalid file format")]
    InvalidFileFormat,
    /// Invalid ROI dimensions or buffer size.
    #[error("invalid size")]
    InvalidSize,
    /// Unsupported image / pixel format.
    #[error("invalid image type")]
    InvalidImageType,
    /// ROI start position is outside the sensor area.
    #[error("out of boundary")]
    OutOfBoundary,
    /// No frame received within the specified wait time.
    #[error("timeout")]
    Timeout,
    /// Operation not allowed while capturing; stop capture first.
    #[error("invalid sequence (stop capture first)")]
    InvalidSequence,
    /// Supplied buffer is too small for the frame data.
    #[error("buffer too small")]
    BufferTooSmall,
    /// Operation not allowed while video mode is active.
    #[error("video mode active")]
    VideoModeActive,
    /// An exposure is currently in progress.
    #[error("exposure in progress")]
    ExposureInProgress,
    /// Catch-all for hardware or parameter errors.
    #[error("general error")]
    GeneralError,
    /// Camera mode value is out of range or unsupported.
    #[error("invalid mode")]
    InvalidMode,
    /// Invalid pulse guide direction.
    #[error("invalid guide direction")]
    InvalidDirection,
    /// Camera sensor type is not recognised by the SDK.
    #[error("unknown sensor type")]
    UnknownSensorType,
    /// An error code not mapped by this crate (possibly from a newer SDK).
    #[error("unknown error code: {0}")]
    Unknown(i32),
}

/// Convenience alias used throughout the `svbony` crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Maps a raw C SDK return code to [`Result`].
pub(crate) fn check(code: c_int) -> Result<()> {
    match code {
        SVB_SUCCESS => Ok(()),
        SVB_ERROR_INVALID_INDEX => Err(Error::InvalidIndex),
        SVB_ERROR_INVALID_ID => Err(Error::InvalidId),
        SVB_ERROR_INVALID_CONTROL_TYPE => Err(Error::InvalidControlType),
        SVB_ERROR_CAMERA_CLOSED => Err(Error::CameraClosed),
        SVB_ERROR_CAMERA_REMOVED => Err(Error::CameraRemoved),
        SVB_ERROR_INVALID_PATH => Err(Error::InvalidPath),
        SVB_ERROR_INVALID_FILEFORMAT => Err(Error::InvalidFileFormat),
        SVB_ERROR_INVALID_SIZE => Err(Error::InvalidSize),
        SVB_ERROR_INVALID_IMGTYPE => Err(Error::InvalidImageType),
        SVB_ERROR_OUTOF_BOUNDARY => Err(Error::OutOfBoundary),
        SVB_ERROR_TIMEOUT => Err(Error::Timeout),
        SVB_ERROR_INVALID_SEQUENCE => Err(Error::InvalidSequence),
        SVB_ERROR_BUFFER_TOO_SMALL => Err(Error::BufferTooSmall),
        SVB_ERROR_VIDEO_MODE_ACTIVE => Err(Error::VideoModeActive),
        SVB_ERROR_EXPOSURE_IN_PROGRESS => Err(Error::ExposureInProgress),
        SVB_ERROR_GENERAL_ERROR => Err(Error::GeneralError),
        SVB_ERROR_INVALID_MODE => Err(Error::InvalidMode),
        SVB_ERROR_INVALID_DIRECTION => Err(Error::InvalidDirection),
        SVB_ERROR_UNKNOW_SENSOR_TYPE => Err(Error::UnknownSensorType),
        other => Err(Error::Unknown(other as i32)),
    }
}
