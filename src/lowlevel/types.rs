#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum SVBBayerPattern {
    SVBBayerRg = 0,
    SVBBayerBg = 1,
    SVBBayerGr = 2,
    SVBBayerGb = 3,
}

impl From<i32> for SVBBayerPattern {
    fn from(value: i32) -> Self {
        match value {
            0 => SVBBayerPattern::SVBBayerRg,
            1 => SVBBayerPattern::SVBBayerBg,
            2 => SVBBayerPattern::SVBBayerGr,
            3 => SVBBayerPattern::SVBBayerGb,
            _ => panic!("Unknown SVBBayerPattern value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(C)]
pub enum SVBImageType {
    SVBImageRaw8 = 0,
    SVBImageRaw10 = 2,
    SVBImageRaw12 = 3,
    SVBImageRaw14 = 4,
    SVBImageRaw16 = 5,
    SVBImageY8 = 6,
    SVBImageY10 = 7,
    SVBImageY12 = 8,
    SVBImageY14 = 9,
    SVBImageY16 = 10,
    SVBImageRGB24 = 11,
    SVBImageRGB32 = 12,
    SVBImageEnd = -1,
}

impl From<i32> for SVBImageType {
    fn from(value: i32) -> Self {
        match value {
            0 => SVBImageType::SVBImageRaw8,
            2 => SVBImageType::SVBImageRaw10,
            3 => SVBImageType::SVBImageRaw12,
            4 => SVBImageType::SVBImageRaw14,
            5 => SVBImageType::SVBImageRaw16,
            6 => SVBImageType::SVBImageY8,
            7 => SVBImageType::SVBImageY10,
            8 => SVBImageType::SVBImageY12,
            9 => SVBImageType::SVBImageY14,
            10 => SVBImageType::SVBImageY16,
            11 => SVBImageType::SVBImageRGB24,
            12 => SVBImageType::SVBImageRGB32,
            -1 => SVBImageType::SVBImageEnd,
            _ => panic!("Unknown SVBImageType value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum SVBGuideDirection {
    SVBGuideNorth = 0,
    SVBGuideSouth = 1,
    SVBGuideEast = 2,
    SVBGuideWest = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum SVBFlipStatus {
    SVBFlipNone = 0,
    SVBFlipHorizontal = 1,
    SVBFlipVertical = 2,
    SVBFlipBoth = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SVBCameraMode {
    SVBCameraModeNormal = 0,
    SVBCameraModeTrigSoft = 1,
    SVBCameraModeTrigRiseEdge = 2,
    SVBCameraModeTrigFallEdge = 3,
    SVBCameraModeTrigDoubleEdge = 4,
    SVBCameraModeTrigHighLevel = 5,
    SVBCameraModeTrigLowLevel = 6,
}

impl From<i32> for SVBCameraMode {
    fn from(value: i32) -> Self {
        match value {
            0 => SVBCameraMode::SVBCameraModeNormal,
            1 => SVBCameraMode::SVBCameraModeTrigSoft,
            2 => SVBCameraMode::SVBCameraModeTrigRiseEdge,
            3 => SVBCameraMode::SVBCameraModeTrigFallEdge,
            4 => SVBCameraMode::SVBCameraModeTrigDoubleEdge,
            5 => SVBCameraMode::SVBCameraModeTrigHighLevel,
            6 => SVBCameraMode::SVBCameraModeTrigLowLevel,
            _ => panic!("Unknown SVBCameraMode value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SVBTrigOuput {
    SVBTrigOutputPinA = 0,
    SVBTrigOutputPinB = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(i32)]
pub enum SVBErrorCode {
    SVBSuccess = 0,
    SVBErrorInvalidIndex = 1,
    SVBErrorInvalidId = 2,
    SVBErrorInvalidControlType = 3,
    SVBErrorCameraClosed = 4,
    SVBErrorCameraRemoved = 5,
    SVBErrorInvalidPath = 6,
    SVBErrorInvalidFileFormat = 7,
    SVBErrorInvalidSize = 8,
    SVBErrorInvalidImageType = 9,
    SVBErrrorOutOfBoundary = 10,
    SVBErrorTimeout = 11,
    SVBErrorInvalidSequence = 12,
    SVBErrorBufferToosmall = 13,
    SVBErrorVideoModeActive = 14,
    SVBErrorExposureInProgress = 15,
    SVBErrorGeneralError = 16,
    SVBErrorInvalidMode = 17,
    SVBErrorInvalidDirection = 18,
    SVBErrorUnknownSensorType = 19,
}

impl From<i32> for SVBErrorCode {
    fn from(value: i32) -> Self {
        match value {
            0 => SVBErrorCode::SVBSuccess,
            1 => SVBErrorCode::SVBErrorInvalidIndex,
            2 => SVBErrorCode::SVBErrorInvalidId,
            3 => SVBErrorCode::SVBErrorInvalidControlType,
            4 => SVBErrorCode::SVBErrorCameraClosed,
            5 => SVBErrorCode::SVBErrorCameraRemoved,
            6 => SVBErrorCode::SVBErrorInvalidPath,
            7 => SVBErrorCode::SVBErrorInvalidFileFormat,
            8 => SVBErrorCode::SVBErrorInvalidSize,
            9 => SVBErrorCode::SVBErrorInvalidImageType,
            10 => SVBErrorCode::SVBErrrorOutOfBoundary,
            11 => SVBErrorCode::SVBErrorTimeout,
            12 => SVBErrorCode::SVBErrorInvalidSequence,
            13 => SVBErrorCode::SVBErrorBufferToosmall,
            14 => SVBErrorCode::SVBErrorVideoModeActive,
            15 => SVBErrorCode::SVBErrorExposureInProgress,
            16 => SVBErrorCode::SVBErrorGeneralError,
            17 => SVBErrorCode::SVBErrorInvalidMode,
            18 => SVBErrorCode::SVBErrorInvalidDirection,
            19 => SVBErrorCode::SVBErrorUnknownSensorType,
            _ => panic!("Unknown error code: {}", value),
        }
    }
}

impl std::error::Error for SVBErrorCode {
    fn description(&self) -> &str {
        match self {
            SVBErrorCode::SVBSuccess => "Success",
            SVBErrorCode::SVBErrorInvalidIndex => "Invalid index",
            SVBErrorCode::SVBErrorInvalidId => "Invalid ID",
            SVBErrorCode::SVBErrorInvalidControlType => "Invalid control type",
            SVBErrorCode::SVBErrorCameraClosed => "Camera closed",
            SVBErrorCode::SVBErrorCameraRemoved => "Camera removed",
            SVBErrorCode::SVBErrorInvalidPath => "Invalid path",
            SVBErrorCode::SVBErrorInvalidFileFormat => "Invalid file format",
            SVBErrorCode::SVBErrorInvalidSize => "Invalid size",
            SVBErrorCode::SVBErrorInvalidImageType => "Invalid image type",
            SVBErrorCode::SVBErrrorOutOfBoundary => "Out of boundary",
            SVBErrorCode::SVBErrorTimeout => "Timeout",
            SVBErrorCode::SVBErrorInvalidSequence => "Invalid sequence",
            SVBErrorCode::SVBErrorBufferToosmall => "Buffer too small",
            SVBErrorCode::SVBErrorVideoModeActive => "Video mode active",
            SVBErrorCode::SVBErrorExposureInProgress => "Exposure in progress",
            SVBErrorCode::SVBErrorGeneralError => "General error",
            SVBErrorCode::SVBErrorInvalidMode => "Invalid mode",
            SVBErrorCode::SVBErrorInvalidDirection => "Invalid direction",
            SVBErrorCode::SVBErrorUnknownSensorType => "Unknown sensor type",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl std::fmt::Display for SVBErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SVBErrorCode::SVBSuccess => write!(f, "Success"),
            SVBErrorCode::SVBErrorInvalidIndex => write!(f, "Invalid index"),
            SVBErrorCode::SVBErrorInvalidId => write!(f, "Invalid ID"),
            SVBErrorCode::SVBErrorInvalidControlType => write!(f, "Invalid control type"),
            SVBErrorCode::SVBErrorCameraClosed => write!(f, "Camera closed"),
            SVBErrorCode::SVBErrorCameraRemoved => write!(f, "Camera removed"),
            SVBErrorCode::SVBErrorInvalidPath => write!(f, "Invalid path"),
            SVBErrorCode::SVBErrorInvalidFileFormat => write!(f, "Invalid file format"),
            SVBErrorCode::SVBErrorInvalidSize => write!(f, "Invalid size"),
            SVBErrorCode::SVBErrorInvalidImageType => write!(f, "Invalid image type"),
            SVBErrorCode::SVBErrrorOutOfBoundary => write!(f, "Out of boundary"),
            SVBErrorCode::SVBErrorTimeout => write!(f, "Timeout"),
            SVBErrorCode::SVBErrorInvalidSequence => write!(f, "Invalid sequence"),
            SVBErrorCode::SVBErrorBufferToosmall => write!(f, "Buffer too small"),
            SVBErrorCode::SVBErrorVideoModeActive => write!(f, "Video mode active"),
            SVBErrorCode::SVBErrorExposureInProgress => write!(f, "Exposure in progress"),
            SVBErrorCode::SVBErrorGeneralError => write!(f, "General error"),
            SVBErrorCode::SVBErrorInvalidMode => write!(f, "Invalid mode"),
            SVBErrorCode::SVBErrorInvalidDirection => write!(f, "Invalid direction"),
            SVBErrorCode::SVBErrorUnknownSensorType => write!(f, "Unknown sensor type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum SVBBool {
    SVBFalse = 0,
    SVBTrue = 1,
}

impl From<SVBBool> for bool {
    fn from(value: SVBBool) -> Self {
        match value {
            SVBBool::SVBFalse => false,
            SVBBool::SVBTrue => true,
        }
    }
}

impl From<i32> for SVBBool {
    fn from(value: i32) -> Self {
        match value {
            0 => SVBBool::SVBFalse,
            1 => SVBBool::SVBTrue,
            _ => panic!("Unknown SVBBool value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SVBControlType {
    SVBGain = 0,
    SVBExposure = 1,
    SVBGamma = 2,
    SVBGammaContrast = 3,
    SVBWbRed = 4,
    SVBWbGreen = 5,
    SVBWbBlue = 6,
    SVBFlip = 7,
    SVBFrameSpeedMode = 8,
    SVBContrast = 9,
    SVBSharpness = 10,
    SVBSaturation = 11,
    SVBAutoTargetBrightness = 12,
    SVBBlackLevel = 13,
    SVBCoolerEnable = 14,
    SVBTargetTemperature = 15,
    SVBCurrentTemperature = 16,
    SVBCoolerPower = 17,
    SVBBadPixelCorrectionEnable = 18,
    SVBBadPixelCorrectionThreshold = 19,
}

impl From<i32> for SVBControlType {
    fn from(value: i32) -> Self {
        match value {
            0 => SVBControlType::SVBGain,
            1 => SVBControlType::SVBExposure,
            2 => SVBControlType::SVBGamma,
            3 => SVBControlType::SVBGammaContrast,
            4 => SVBControlType::SVBWbRed,
            5 => SVBControlType::SVBWbGreen,
            6 => SVBControlType::SVBWbBlue,
            7 => SVBControlType::SVBFlip,
            8 => SVBControlType::SVBFrameSpeedMode,
            9 => SVBControlType::SVBContrast,
            10 => SVBControlType::SVBSharpness,
            11 => SVBControlType::SVBSaturation,
            12 => SVBControlType::SVBAutoTargetBrightness,
            13 => SVBControlType::SVBBlackLevel,
            14 => SVBControlType::SVBCoolerEnable,
            15 => SVBControlType::SVBTargetTemperature,
            16 => SVBControlType::SVBCurrentTemperature,
            17 => SVBControlType::SVBCoolerPower,
            18 => SVBControlType::SVBBadPixelCorrectionEnable,
            19 => SVBControlType::SVBBadPixelCorrectionThreshold,
            _ => panic!("Unknown SVBControlType value: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SVBExposureStatus {
    SVBExposureIdle = 0,
    SVBExposureInProgress = 1,
    SVBExposureSuccess = 2,
    SVBExposureFailed = 3,
}
