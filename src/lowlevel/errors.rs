/// Error types for the high-level API.
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum SVBError {
    #[error("Success")]
    Success,
    #[error("Invalid index")]
    InvalidIndex,
    #[error("Invalid ID")]
    InvalidId,
    #[error("Invalid control type")]
    InvalidControlType,
    #[error("Camera closed")]
    CameraClosed,
    #[error("Camera removed")]
    CameraRemoved,
    #[error("Invalid path")]
    InvalidPath,
    #[error("Invalid file format")]
    InvalidFileFormat,
    #[error("Invalid size")]
    InvalidSize,
    #[error("Invalid image type")]
    InvalidImageType,
    #[error("Out of boundary")]
    OutOfBoundary,
    #[error("Timeout")]
    Timeout,
    #[error("Invalid sequence")]
    InvalidSequence,
    #[error("Buffer too small")]
    BufferTooSmall,
    #[error("Video mode active")]
    VideoModeActive,
    #[error("Exposure in progress")]
    ExposureInProgress,
    #[error("General error")]
    GeneralError,
    #[error("Invalid mode")]
    InvalidMode,
    #[error("Invalid direction")]
    InvalidDirection,
    #[error("Unknown sensor type")]
    UnknownSensorType,
}

impl From<SVBError> for Result<(), SVBError> {
    fn from(error: SVBError) -> Self {
        match error {
            SVBError::Success => Ok(()),
            _ => Err(error),
        }
    }
}

impl From<std::ffi::c_int> for SVBError {
    fn from(code: std::ffi::c_int) -> Self {
        match code {
            0 => SVBError::Success,
            1 => SVBError::InvalidIndex,
            2 => SVBError::InvalidId,
            3 => SVBError::InvalidControlType,
            4 => SVBError::CameraClosed,
            5 => SVBError::CameraRemoved,
            6 => SVBError::InvalidPath,
            7 => SVBError::InvalidFileFormat,
            8 => SVBError::InvalidSize,
            9 => SVBError::InvalidImageType,
            10 => SVBError::OutOfBoundary,
            11 => SVBError::Timeout,
            12 => SVBError::InvalidSequence,
            13 => SVBError::BufferTooSmall,
            14 => SVBError::VideoModeActive,
            15 => SVBError::ExposureInProgress,
            16 => SVBError::GeneralError,
            17 => SVBError::InvalidMode,
            18 => SVBError::InvalidDirection,
            19 => SVBError::UnknownSensorType,
            _ => SVBError::GeneralError, // Fallback for unknown error codes
        }
    }
}
