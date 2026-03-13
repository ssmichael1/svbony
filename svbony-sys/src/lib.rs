//! Raw FFI bindings to the SVBony USB camera C SDK (v1.13.4).
//!
//! This crate provides unsafe, low-level access to all SDK functions and types.
//! For a safe, idiomatic Rust API, use the [`svbony`](https://crates.io/crates/svbony)
//! crate instead.
//!
//! # SDK Installation
//!
//! The SVBony Camera SDK is closed-source and **not bundled** with this crate.
//! Download it from [SVBony's website](https://www.svbony.com/), then set the
//! `SVBCAMERA_SDK_PATH` environment variable to the SDK root directory (the one
//! containing `include/` and `lib/`):
//!
//! ```sh
//! export SVBCAMERA_SDK_PATH=/path/to/SVBCameraSDK
//! cargo build
//! ```
//!
//! # Platform Support
//!
//! | Target | Arch | SDK subdir |
//! |--------|------|------------|
//! | macOS | aarch64 | `lib/arm64` |
//! | macOS | x86_64 | `lib/x64` |
//! | Linux | x86_64 | `lib/x64` |
//! | Linux | x86 | `lib/x86` |
//! | Linux | armv7 | `lib/armv7` |
//! | Linux | aarch64 | `lib/armv8` |
//! | Windows | x86_64 | `lib/x64` |
//! | Windows | x86 | `lib/x86` |
//!
//! On macOS and Linux the SDK is linked statically (`libSVBCameraSDK.a`) with a
//! dynamic dependency on `libusb-1.0`. On Windows the SDK ships as a DLL.
//!
//! # Safety
//!
//! All functions in the `extern "C"` block are unsafe. Callers must ensure that
//! pointer arguments are valid and that buffers are large enough. The
//! [`svbony`](https://crates.io/crates/svbony) crate wraps these in a safe API.

#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::{c_char, c_float, c_int, c_long, c_uchar, c_uint};

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Basic camera identification, returned before the camera is opened.
///
/// Obtained via [`SVBGetCameraInfo`].
#[repr(C)]
pub struct SVB_CAMERA_INFO {
    /// Human-readable camera name (e.g. "SV305 Pro").
    pub FriendlyName: [c_char; 32],
    /// Serial number string.
    pub CameraSN: [c_char; 32],
    /// Connection type (e.g. "USB3.0").
    pub PortType: [c_char; 32],
    /// OS-level device identifier.
    pub DeviceID: c_uint,
    /// SDK camera handle used in all subsequent calls.
    pub CameraID: c_int,
}

/// Static camera properties (sensor size, color, supported modes).
///
/// Obtained via [`SVBGetCameraProperty`]. The camera does **not** need to be
/// open to query this.
#[repr(C)]
pub struct SVB_CAMERA_PROPERTY {
    /// Maximum sensor height in pixels.
    pub MaxHeight: c_long,
    /// Maximum sensor width in pixels.
    pub MaxWidth: c_long,
    /// Non-zero if the camera has a color sensor.
    pub IsColorCam: c_int,
    /// Bayer filter arrangement (see `SVB_BAYER_*` constants). Only meaningful
    /// when `IsColorCam` is non-zero.
    pub BayerPattern: c_int,
    /// Supported binning factors, zero-terminated (e.g. `[1, 2, 0, ...]`).
    pub SupportedBins: [c_int; 16],
    /// Supported output pixel formats, terminated by [`SVB_IMG_END`].
    pub SupportedVideoFormat: [c_int; 8],
    /// Maximum ADC bit depth.
    pub MaxBitDepth: c_int,
    /// Non-zero if the camera supports external/software trigger modes.
    pub IsTriggerCam: c_int,
}

/// Extended camera properties.
///
/// Obtained via [`SVBGetCameraPropertyEx`].
#[repr(C)]
pub struct SVB_CAMERA_PROPERTY_EX {
    /// Non-zero if the camera supports pulse guiding (ST4).
    pub bSupportPulseGuide: c_int,
    /// Non-zero if the camera has a temperature sensor / TEC.
    pub bSupportControlTemp: c_int,
    /// Reserved for future use.
    pub Unused: [c_int; 64],
}

/// Describes a single camera control (gain, exposure, etc.).
///
/// Obtained via [`SVBGetControlCaps`].
#[repr(C)]
pub struct SVB_CONTROL_CAPS {
    /// Control name (e.g. "Exposure", "Gain").
    pub Name: [c_char; 64],
    /// Human-readable description.
    pub Description: [c_char; 128],
    /// Maximum allowed value.
    pub MaxValue: c_long,
    /// Minimum allowed value.
    pub MinValue: c_long,
    /// Factory default value.
    pub DefaultValue: c_long,
    /// Non-zero if auto mode is supported for this control.
    pub IsAutoSupported: c_int,
    /// Non-zero if the control can be written (some, like temperature, are read-only).
    pub IsWritable: c_int,
    /// Control type identifier (see `SVB_GAIN`, `SVB_EXPOSURE`, etc.).
    pub ControlType: c_int,
    /// Reserved.
    pub Unused: [c_char; 32],
}

/// 64-byte camera serial number / unique identifier.
///
/// Obtained via [`SVBGetSerialNumber`].
#[repr(C)]
pub struct SVB_ID {
    pub id: [c_uchar; 64],
}

/// List of camera modes supported by a trigger-capable camera.
///
/// Obtained via [`SVBGetCameraSupportMode`]. The array is terminated by
/// [`SVB_MODE_END`].
#[repr(C)]
pub struct SVB_SUPPORTED_MODE {
    pub SupportedCameraMode: [c_int; 16],
}

// ---------------------------------------------------------------------------
// Error codes (SVB_ERROR_CODE)
// ---------------------------------------------------------------------------

/// Operation completed successfully.
pub const SVB_SUCCESS: c_int = 0;
/// No camera connected or camera index out of range.
pub const SVB_ERROR_INVALID_INDEX: c_int = 1;
/// No camera with this ID is connected.
pub const SVB_ERROR_INVALID_ID: c_int = 2;
/// Invalid control type passed to get/set control.
pub const SVB_ERROR_INVALID_CONTROL_TYPE: c_int = 3;
/// Camera has not been opened yet.
pub const SVB_ERROR_CAMERA_CLOSED: c_int = 4;
/// Camera was disconnected (USB removed).
pub const SVB_ERROR_CAMERA_REMOVED: c_int = 5;
/// File path does not exist.
pub const SVB_ERROR_INVALID_PATH: c_int = 6;
/// Unsupported file format.
pub const SVB_ERROR_INVALID_FILEFORMAT: c_int = 7;
/// Invalid ROI dimensions or buffer size.
pub const SVB_ERROR_INVALID_SIZE: c_int = 8;
/// Unsupported image / pixel format.
pub const SVB_ERROR_INVALID_IMGTYPE: c_int = 9;
/// ROI start position is outside the sensor area.
pub const SVB_ERROR_OUTOF_BOUNDARY: c_int = 10;
/// Frame capture timed out.
pub const SVB_ERROR_TIMEOUT: c_int = 11;
/// Operation not allowed while capturing (stop capture first).
pub const SVB_ERROR_INVALID_SEQUENCE: c_int = 12;
/// Supplied buffer is too small for the frame data.
pub const SVB_ERROR_BUFFER_TOO_SMALL: c_int = 13;
/// Cannot perform this operation while video mode is active.
pub const SVB_ERROR_VIDEO_MODE_ACTIVE: c_int = 14;
/// An exposure is currently in progress.
pub const SVB_ERROR_EXPOSURE_IN_PROGRESS: c_int = 15;
/// Catch-all for hardware or parameter errors.
pub const SVB_ERROR_GENERAL_ERROR: c_int = 16;
/// Camera mode value is out of range or unsupported.
pub const SVB_ERROR_INVALID_MODE: c_int = 17;
/// Invalid pulse guide direction.
pub const SVB_ERROR_INVALID_DIRECTION: c_int = 18;
/// Camera sensor type is not recognised by the SDK.
pub const SVB_ERROR_UNKNOW_SENSOR_TYPE: c_int = 19;
/// Sentinel (not a real error code).
pub const SVB_ERROR_END: c_int = 20;

// ---------------------------------------------------------------------------
// Image types (SVB_IMG_TYPE)
// ---------------------------------------------------------------------------

/// 8-bit raw Bayer data.
pub const SVB_IMG_RAW8: c_int = 0;
/// 10-bit raw Bayer in 16-bit container.
pub const SVB_IMG_RAW10: c_int = 1;
/// 12-bit raw Bayer in 16-bit container.
pub const SVB_IMG_RAW12: c_int = 2;
/// 14-bit raw Bayer in 16-bit container.
pub const SVB_IMG_RAW14: c_int = 3;
/// 16-bit raw Bayer.
pub const SVB_IMG_RAW16: c_int = 4;
/// 8-bit mono (debayered luminance).
pub const SVB_IMG_Y8: c_int = 5;
/// 10-bit mono in 16-bit container.
pub const SVB_IMG_Y10: c_int = 6;
/// 12-bit mono in 16-bit container.
pub const SVB_IMG_Y12: c_int = 7;
/// 14-bit mono in 16-bit container.
pub const SVB_IMG_Y14: c_int = 8;
/// 16-bit mono.
pub const SVB_IMG_Y16: c_int = 9;
/// 24-bit RGB (3 bytes per pixel).
pub const SVB_IMG_RGB24: c_int = 10;
/// 32-bit RGBA (4 bytes per pixel).
pub const SVB_IMG_RGB32: c_int = 11;
/// Sentinel marking the end of the supported format list.
pub const SVB_IMG_END: c_int = -1;

// ---------------------------------------------------------------------------
// Bayer patterns (SVB_BAYER_PATTERN)
// ---------------------------------------------------------------------------

/// RGGB Bayer pattern.
pub const SVB_BAYER_RG: c_int = 0;
/// BGGR Bayer pattern.
pub const SVB_BAYER_BG: c_int = 1;
/// GRBG Bayer pattern.
pub const SVB_BAYER_GR: c_int = 2;
/// GBRG Bayer pattern.
pub const SVB_BAYER_GB: c_int = 3;

// ---------------------------------------------------------------------------
// Control types (SVB_CONTROL_TYPE)
// ---------------------------------------------------------------------------

/// Sensor gain.
pub const SVB_GAIN: c_int = 0;
/// Exposure time in microseconds.
pub const SVB_EXPOSURE: c_int = 1;
/// Gamma correction.
pub const SVB_GAMMA: c_int = 2;
/// Gamma contrast.
pub const SVB_GAMMA_CONTRAST: c_int = 3;
/// White balance red channel.
pub const SVB_WB_R: c_int = 4;
/// White balance green channel.
pub const SVB_WB_G: c_int = 5;
/// White balance blue channel.
pub const SVB_WB_B: c_int = 6;
/// Image flip (see `SVB_FLIP_*` constants).
pub const SVB_FLIP: c_int = 7;
/// Frame readout speed: 0 = low, 1 = medium, 2 = high.
pub const SVB_FRAME_SPEED_MODE: c_int = 8;
/// Contrast adjustment.
pub const SVB_CONTRAST: c_int = 9;
/// Sharpness adjustment.
pub const SVB_SHARPNESS: c_int = 10;
/// Saturation adjustment (color cameras only).
pub const SVB_SATURATION: c_int = 11;
/// Auto-exposure target brightness.
pub const SVB_AUTO_TARGET_BRIGHTNESS: c_int = 12;
/// Black level (pedestal) offset.
pub const SVB_BLACK_LEVEL: c_int = 13;
/// TEC cooler enable (0 = off, 1 = on).
pub const SVB_COOLER_ENABLE: c_int = 14;
/// TEC target temperature in units of 0.1 C.
pub const SVB_TARGET_TEMPERATURE: c_int = 15;
/// Current sensor temperature in units of 0.1 C (read-only).
pub const SVB_CURRENT_TEMPERATURE: c_int = 16;
/// Current TEC cooler power percentage, 0-100 (read-only).
pub const SVB_COOLER_POWER: c_int = 17;
/// Bad pixel correction enable (0 = off, 1 = on).
pub const SVB_BAD_PIXEL_CORRECTION_ENABLE: c_int = 18;
/// Bad pixel correction sensitivity threshold.
pub const SVB_BAD_PIXEL_CORRECTION_THRESHOLD: c_int = 19;

// ---------------------------------------------------------------------------
// Camera modes (SVB_CAMERA_MODE)
// ---------------------------------------------------------------------------

/// Normal free-running capture mode.
pub const SVB_MODE_NORMAL: c_int = 0;
/// Software trigger mode.
pub const SVB_MODE_TRIG_SOFT: c_int = 1;
/// Hardware trigger on rising edge.
pub const SVB_MODE_TRIG_RISE_EDGE: c_int = 2;
/// Hardware trigger on falling edge.
pub const SVB_MODE_TRIG_FALL_EDGE: c_int = 3;
/// Hardware trigger on both edges.
pub const SVB_MODE_TRIG_DOUBLE_EDGE: c_int = 4;
/// Hardware trigger on high level.
pub const SVB_MODE_TRIG_HIGH_LEVEL: c_int = 5;
/// Hardware trigger on low level.
pub const SVB_MODE_TRIG_LOW_LEVEL: c_int = 6;
/// Sentinel marking end of supported mode list.
pub const SVB_MODE_END: c_int = -1;

// ---------------------------------------------------------------------------
// Guide directions (SVB_GUIDE_DIRECTION)
// ---------------------------------------------------------------------------

/// Pulse guide north (declination +).
pub const SVB_GUIDE_NORTH: c_int = 0;
/// Pulse guide south (declination -).
pub const SVB_GUIDE_SOUTH: c_int = 1;
/// Pulse guide east (right ascension +).
pub const SVB_GUIDE_EAST: c_int = 2;
/// Pulse guide west (right ascension -).
pub const SVB_GUIDE_WEST: c_int = 3;

// ---------------------------------------------------------------------------
// Flip status (SVB_FLIP_STATUS)
// ---------------------------------------------------------------------------

/// No flip (original orientation).
pub const SVB_FLIP_NONE: c_int = 0;
/// Horizontal flip (mirror).
pub const SVB_FLIP_HORIZ: c_int = 1;
/// Vertical flip.
pub const SVB_FLIP_VERT: c_int = 2;
/// Both horizontal and vertical flip (180-degree rotation).
pub const SVB_FLIP_BOTH: c_int = 3;

// ---------------------------------------------------------------------------
// Trigger output pins (SVB_TRIG_OUTPUT_PIN)
// ---------------------------------------------------------------------------

/// Trigger output on Pin A.
pub const SVB_TRIG_OUTPUT_PINA: c_int = 0;
/// Trigger output on Pin B.
pub const SVB_TRIG_OUTPUT_PINB: c_int = 1;
/// No trigger output.
pub const SVB_TRIG_OUTPUT_NONE: c_int = -1;

// ---------------------------------------------------------------------------
// Exposure status (SVB_EXPOSURE_STATUS)
// ---------------------------------------------------------------------------

/// Idle, ready to start a new exposure.
pub const SVB_EXP_IDLE: c_int = 0;
/// Exposure in progress.
pub const SVB_EXP_WORKING: c_int = 1;
/// Exposure completed successfully, data ready for download.
pub const SVB_EXP_SUCCESS: c_int = 2;
/// Exposure failed.
pub const SVB_EXP_FAILED: c_int = 3;

// ---------------------------------------------------------------------------
// Boolean (SVB_BOOL)
// ---------------------------------------------------------------------------

/// False.
pub const SVB_FALSE: c_int = 0;
/// True.
pub const SVB_TRUE: c_int = 1;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

extern "C" {
    /// Returns the number of connected SVBony cameras.
    ///
    /// This should be the first SDK function called. The return value is used
    /// as the upper bound for the `iCameraIndex` parameter of
    /// [`SVBGetCameraInfo`].
    pub fn SVBGetNumOfConnectedCameras() -> c_int;

    /// Populates `pSVBCameraInfo` with identification data for the camera at
    /// `iCameraIndex` (0-based). Does not require the camera to be open.
    pub fn SVBGetCameraInfo(pSVBCameraInfo: *mut SVB_CAMERA_INFO, iCameraIndex: c_int) -> c_int;

    /// Retrieves static sensor properties for the camera identified by
    /// `iCameraID`.
    pub fn SVBGetCameraProperty(
        iCameraID: c_int,
        pCameraProperty: *mut SVB_CAMERA_PROPERTY,
    ) -> c_int;

    /// Retrieves extended properties (pulse guide support, temperature control)
    /// for `iCameraID`.
    pub fn SVBGetCameraPropertyEx(
        iCameraID: c_int,
        pCameraPropertyEx: *mut SVB_CAMERA_PROPERTY_EX,
    ) -> c_int;

    /// Opens the camera for use. Must be called before any control, capture,
    /// or mode operations. Does not affect a camera that is already capturing.
    pub fn SVBOpenCamera(iCameraID: c_int) -> c_int;

    /// Closes the camera and releases its resources. Returns [`SVB_SUCCESS`]
    /// even if the camera is already closed.
    pub fn SVBCloseCamera(iCameraID: c_int) -> c_int;

    /// Returns the number of adjustable controls for an opened camera.
    pub fn SVBGetNumOfControls(iCameraID: c_int, piNumberOfControls: *mut c_int) -> c_int;

    /// Retrieves the capabilities of the control at `iControlIndex` (0-based,
    /// **not** the control type). The camera must be open.
    pub fn SVBGetControlCaps(
        iCameraID: c_int,
        iControlIndex: c_int,
        pControlCaps: *mut SVB_CONTROL_CAPS,
    ) -> c_int;

    /// Reads the current value and auto-mode flag for a given `ControlType`.
    ///
    /// Temperature values are reported as `float * 10` cast to `long`.
    pub fn SVBGetControlValue(
        iCameraID: c_int,
        ControlType: c_int,
        plValue: *mut c_long,
        pbAuto: *mut c_int,
    ) -> c_int;

    /// Sets a control value and auto-mode flag. Values outside the valid range
    /// are clamped to min/max and [`SVB_SUCCESS`] is still returned.
    pub fn SVBSetControlValue(
        iCameraID: c_int,
        ControlType: c_int,
        lValue: c_long,
        bAuto: c_int,
    ) -> c_int;

    /// Gets the current output image pixel format.
    pub fn SVBGetOutputImageType(iCameraID: c_int, pImageType: *mut c_int) -> c_int;

    /// Sets the output image pixel format. The value must be one of the formats
    /// listed in [`SVB_CAMERA_PROPERTY::SupportedVideoFormat`].
    pub fn SVBSetOutputImageType(iCameraID: c_int, ImageType: c_int) -> c_int;

    /// Configures the region of interest (ROI) and binning. Capture must be
    /// stopped before calling this.
    ///
    /// Width and height are post-binning dimensions. Width must be a multiple
    /// of 8, height a multiple of 2.
    pub fn SVBSetROIFormat(
        iCameraID: c_int,
        iStartX: c_int,
        iStartY: c_int,
        iWidth: c_int,
        iHeight: c_int,
        iBin: c_int,
    ) -> c_int;

    /// Like [`SVBSetROIFormat`] but also selects the binning mode
    /// (`iMode`: 0 = average, 1 = sum).
    pub fn SVBSetROIFormatEx(
        iCameraID: c_int,
        iStartX: c_int,
        iStartY: c_int,
        iWidth: c_int,
        iHeight: c_int,
        iBin: c_int,
        iMode: c_int,
    ) -> c_int;

    /// Reads back the current ROI and binning settings.
    pub fn SVBGetROIFormat(
        iCameraID: c_int,
        piStartX: *mut c_int,
        piStartY: *mut c_int,
        piWidth: *mut c_int,
        piHeight: *mut c_int,
        piBin: *mut c_int,
    ) -> c_int;

    /// Like [`SVBGetROIFormat`] but also returns the binning mode.
    pub fn SVBGetROIFormatEx(
        iCameraID: c_int,
        piStartX: *mut c_int,
        piStartY: *mut c_int,
        piWidth: *mut c_int,
        piHeight: *mut c_int,
        piBin: *mut c_int,
        piMode: *mut c_int,
    ) -> c_int;

    /// Returns the number of frames dropped since capture started (resets to 0
    /// on stop).
    pub fn SVBGetDroppedFrames(iCameraID: c_int, piDropFrames: *mut c_int) -> c_int;

    /// Begins continuous video capture. Frames are retrieved with
    /// [`SVBGetVideoData`].
    pub fn SVBStartVideoCapture(iCameraID: c_int) -> c_int;

    /// Stops video capture.
    pub fn SVBStopVideoCapture(iCameraID: c_int) -> c_int;

    /// Reads one frame from the capture buffer into `pBuffer`.
    ///
    /// `lBuffSize` is the buffer size in bytes. `iWaitms` is the timeout in
    /// milliseconds (-1 = wait forever). Recommended timeout is
    /// `exposure * 2 + 500`.
    ///
    /// The required buffer size depends on the output format:
    /// - 8-bit mono: `width * height`
    /// - 16-bit mono: `width * height * 2`
    /// - RGB24: `width * height * 3`
    /// - RGB32: `width * height * 4`
    pub fn SVBGetVideoData(
        iCameraID: c_int,
        pBuffer: *mut c_uchar,
        lBuffSize: c_long,
        iWaitms: c_int,
    ) -> c_int;

    /// Performs a one-shot white balance. On success, read back `SVB_WB_R`,
    /// `SVB_WB_G`, `SVB_WB_B` to update the UI.
    pub fn SVBWhiteBalanceOnce(iCameraID: c_int) -> c_int;

    /// Retrieves the camera firmware version string. The buffer must be at
    /// least 64 bytes.
    pub fn SVBGetCameraFirmwareVersion(
        iCameraID: c_int,
        pCameraFirmwareVersion: *mut c_char,
    ) -> c_int;

    /// Returns a pointer to a static SDK version string (e.g. `"1, 13, 0503"`).
    pub fn SVBGetSDKVersion() -> *const c_char;

    /// Gets the list of camera modes supported by a trigger-capable camera.
    pub fn SVBGetCameraSupportMode(
        iCameraID: c_int,
        pSupportedMode: *mut SVB_SUPPORTED_MODE,
    ) -> c_int;

    /// Gets the current camera mode (normal or one of the trigger modes).
    pub fn SVBGetCameraMode(iCameraID: c_int, mode: *mut c_int) -> c_int;

    /// Sets the camera mode. Capture must be stopped first.
    pub fn SVBSetCameraMode(iCameraID: c_int, mode: c_int) -> c_int;

    /// Sends a software trigger. For edge trigger, this starts exposure. For
    /// level trigger, call once to start and once to stop.
    pub fn SVBSendSoftTrigger(iCameraID: c_int) -> c_int;

    /// Reads the camera's 64-byte serial number.
    pub fn SVBGetSerialNumber(iCameraID: c_int, pSN: *mut SVB_ID) -> c_int;

    /// Configures a trigger output pin. If `lDuration <= 0` the pin is
    /// disabled.
    ///
    /// `lDelay` and `lDuration` are in microseconds (0 to 2 000 000 000).
    pub fn SVBSetTriggerOutputIOConf(
        iCameraID: c_int,
        pin: c_int,
        bPinHigh: c_int,
        lDelay: c_long,
        lDuration: c_long,
    ) -> c_int;

    /// Reads the current trigger output pin configuration.
    pub fn SVBGetTriggerOutputIOConf(
        iCameraID: c_int,
        pin: c_int,
        bPinHigh: *mut c_int,
        lDelay: *mut c_long,
        lDuration: *mut c_long,
    ) -> c_int;

    /// Sends a pulse guide command (ST4) to the telescope mount.
    ///
    /// `duration` is in milliseconds.
    pub fn SVBPulseGuide(iCameraID: c_int, direction: c_int, duration: c_int) -> c_int;

    /// Returns the sensor pixel size in microns.
    pub fn SVBGetSensorPixelSize(iCameraID: c_int, fPixelSize: *mut c_float) -> c_int;

    /// Checks whether the camera supports pulse guiding (ST4).
    pub fn SVBCanPulseGuide(iCameraID: c_int, pCanPulseGuide: *mut c_int) -> c_int;

    /// Enables or disables automatic saving of camera parameters to a file.
    pub fn SVBSetAutoSaveParam(iCameraID: c_int, enable: c_int) -> c_int;

    /// Checks whether the camera firmware needs upgrading. If so,
    /// `pNeedToUpgradeMinVersion` (at least 64 bytes) is filled with the
    /// minimum required version string.
    pub fn SVBIsCameraNeedToUpgrade(
        iCameraID: c_int,
        pIsNeedToUpgrade: *mut c_int,
        pNeedToUpgradeMinVersion: *mut c_char,
    ) -> c_int;

    /// Restores all camera parameters to factory defaults.
    pub fn SVBRestoreDefaultParam(iCameraID: c_int) -> c_int;
}
