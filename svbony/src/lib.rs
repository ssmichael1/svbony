//! Safe, idiomatic Rust wrapper for the SVBony USB camera SDK.
//!
//! This crate wraps the C SDK in a safe API with RAII camera handles,
//! Rust enums, and proper error handling. For raw FFI access, see
//! [`svbony_sys`].
//!
//! # Setup
//!
//! The SVBony Camera SDK is closed-source and not bundled with this crate.
//! Download it from [SVBony's website](https://www.svbony.com/), then point
//! the build at it:
//!
//! ```sh
//! export SVBCAMERA_SDK_PATH=/path/to/SVBCameraSDK
//! cargo build
//! ```
//!
//! # Quick Start
//!
//! ```no_run
//! use svbony::{Camera, connected_cameras};
//!
//! let cameras = connected_cameras().unwrap();
//! for info in &cameras {
//!     println!("{}: {}", info.camera_id, info.name);
//! }
//!
//! if let Some(info) = cameras.first() {
//!     let cam = Camera::open(info.camera_id).unwrap();
//!     let prop = cam.property().unwrap();
//!     println!("{}x{}", prop.max_width, prop.max_height);
//! }
//! ```
//!
//! # Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `image` | off | Adds [`Camera::get_image`] which returns an [`image::DynamicImage`]. |

mod error;
mod types;

pub use error::{Error, Result};
pub use types::*;

use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int, c_long};

use error::check;

#[cfg(feature = "image")]
use image::{DynamicImage, GrayImage, ImageBuffer, Rgb, Rgba};

/// Returns the SDK version string (e.g. `"1, 13, 0503"`).
pub fn sdk_version() -> String {
    unsafe {
        let ptr = svbony_sys::SVBGetSDKVersion();
        if ptr.is_null() {
            return String::new();
        }
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

/// Returns info for all currently connected SVBony cameras.
///
/// This does not open any camera; it only queries identification data. Use
/// [`CameraInfo::camera_id`] with [`Camera::open`] to start working with a
/// specific camera.
pub fn connected_cameras() -> Result<Vec<CameraInfo>> {
    let count = unsafe { svbony_sys::SVBGetNumOfConnectedCameras() };
    let mut cameras = Vec::with_capacity(count as usize);
    for i in 0..count {
        let mut info = MaybeUninit::uninit();
        check(unsafe { svbony_sys::SVBGetCameraInfo(info.as_mut_ptr(), i) })?;
        cameras.push(CameraInfo::from(unsafe { &info.assume_init() }));
    }
    Ok(cameras)
}

/// RAII handle to an opened SVBony camera.
///
/// Created via [`Camera::open`]. The camera is automatically closed when this
/// value is dropped. All methods take `&self` since the underlying SDK
/// serialises access internally.
///
/// # Typical Workflow
///
/// ```no_run
/// use svbony::*;
///
/// let cam = Camera::open(0)?;
///
/// // Query properties
/// let prop = cam.property()?;
/// println!("Sensor: {}x{}", prop.max_width, prop.max_height);
///
/// // Configure
/// cam.set_roi(&RoiFormat { start_x: 0, start_y: 0, width: 1280, height: 960, bin: 1 })?;
/// cam.set_output_image_type(ImageType::Raw16)?;
/// cam.set_control(ControlType::Exposure, 10_000, false)?;
///
/// // Capture
/// cam.start_capture()?;
/// let mut buf = vec![0u8; 1280 * 960 * 2];
/// cam.get_frame(&mut buf, 5000)?;
/// cam.stop_capture()?;
/// # Ok::<(), svbony::Error>(())
/// ```
pub struct Camera {
    id: c_int,
}

impl Camera {
    /// Opens a camera by its ID (from [`CameraInfo::camera_id`]).
    ///
    /// The camera must not already be open. Returns an RAII handle that closes
    /// the camera on drop.
    pub fn open(camera_id: i32) -> Result<Self> {
        check(unsafe { svbony_sys::SVBOpenCamera(camera_id as c_int) })?;
        Ok(Self {
            id: camera_id as c_int,
        })
    }

    /// Returns the SDK camera ID.
    pub fn id(&self) -> i32 {
        self.id as i32
    }

    // --- Property queries ---

    /// Returns static sensor properties (resolution, color, supported formats).
    pub fn property(&self) -> Result<CameraProperty> {
        let mut prop = MaybeUninit::uninit();
        check(unsafe { svbony_sys::SVBGetCameraProperty(self.id, prop.as_mut_ptr()) })?;
        Ok(CameraProperty::from(unsafe { &prop.assume_init() }))
    }

    /// Returns extended properties (pulse guide support, temperature control).
    pub fn property_ex(&self) -> Result<CameraPropertyEx> {
        let mut prop = MaybeUninit::uninit();
        check(unsafe { svbony_sys::SVBGetCameraPropertyEx(self.id, prop.as_mut_ptr()) })?;
        Ok(CameraPropertyEx::from(unsafe { &prop.assume_init() }))
    }

    /// Returns the camera firmware version string.
    pub fn firmware_version(&self) -> Result<String> {
        let mut buf = [0i8; 64];
        check(unsafe {
            svbony_sys::SVBGetCameraFirmwareVersion(self.id, buf.as_mut_ptr() as *mut c_char)
        })?;
        Ok(unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_string_lossy()
            .into_owned())
    }

    /// Returns the 64-byte camera serial number.
    pub fn serial_number(&self) -> Result<[u8; 64]> {
        let mut sn = MaybeUninit::<svbony_sys::SVB_ID>::uninit();
        check(unsafe { svbony_sys::SVBGetSerialNumber(self.id, sn.as_mut_ptr()) })?;
        Ok(unsafe { sn.assume_init() }.id)
    }

    /// Returns the sensor pixel size in microns.
    pub fn pixel_size(&self) -> Result<f32> {
        let mut size = 0.0f32;
        check(unsafe { svbony_sys::SVBGetSensorPixelSize(self.id, &mut size) })?;
        Ok(size)
    }

    /// Returns the list of camera modes supported by this camera.
    ///
    /// Only meaningful for trigger-capable cameras
    /// ([`CameraProperty::is_trigger_cam`]).
    pub fn supported_modes(&self) -> Result<Vec<CameraMode>> {
        let mut modes = MaybeUninit::<svbony_sys::SVB_SUPPORTED_MODE>::uninit();
        check(unsafe { svbony_sys::SVBGetCameraSupportMode(self.id, modes.as_mut_ptr()) })?;
        let modes = unsafe { modes.assume_init() };
        Ok(modes
            .SupportedCameraMode
            .iter()
            .copied()
            .take_while(|&m| m != svbony_sys::SVB_MODE_END)
            .filter_map(|m| CameraMode::try_from(m).ok())
            .collect())
    }

    /// Checks whether the camera firmware needs upgrading.
    ///
    /// Returns `(needs_upgrade, minimum_version_string)`.
    pub fn needs_upgrade(&self) -> Result<(bool, String)> {
        let mut need: c_int = 0;
        let mut ver = [0i8; 64];
        check(unsafe {
            svbony_sys::SVBIsCameraNeedToUpgrade(
                self.id,
                &mut need,
                ver.as_mut_ptr() as *mut c_char,
            )
        })?;
        let version = unsafe { CStr::from_ptr(ver.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        Ok((need != 0, version))
    }

    // --- Controls ---

    /// Returns the number of adjustable controls exposed by this camera.
    pub fn num_controls(&self) -> Result<usize> {
        let mut num: c_int = 0;
        check(unsafe { svbony_sys::SVBGetNumOfControls(self.id, &mut num) })?;
        Ok(num as usize)
    }

    /// Returns the capabilities of the control at `index` (0-based).
    ///
    /// Note: `index` is a sequential index, **not** a [`ControlType`] value.
    /// Use [`ControlCaps::control_type`] to find out which control it is.
    pub fn control_caps(&self, index: usize) -> Result<ControlCaps> {
        let mut caps = MaybeUninit::uninit();
        check(unsafe {
            svbony_sys::SVBGetControlCaps(self.id, index as c_int, caps.as_mut_ptr())
        })?;
        Ok(ControlCaps::from(unsafe { &caps.assume_init() }))
    }

    /// Reads the current value and auto-mode flag for a control.
    ///
    /// Returns `(value, is_auto)`. Temperature values are reported as
    /// `float * 10` (e.g. 250 = 25.0 C).
    pub fn get_control(&self, ctrl: ControlType) -> Result<(i64, bool)> {
        let mut value: c_long = 0;
        let mut auto_: c_int = 0;
        check(unsafe {
            svbony_sys::SVBGetControlValue(self.id, ctrl as c_int, &mut value, &mut auto_)
        })?;
        Ok((value as i64, auto_ != 0))
    }

    /// Sets a control value and auto-mode flag.
    ///
    /// Values outside the valid range are clamped by the SDK.
    pub fn set_control(&self, ctrl: ControlType, value: i64, auto_: bool) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBSetControlValue(
                self.id,
                ctrl as c_int,
                value as c_long,
                if auto_ { svbony_sys::SVB_TRUE } else { svbony_sys::SVB_FALSE },
            )
        })
    }

    // --- Image format ---

    /// Returns the current output pixel format.
    pub fn output_image_type(&self) -> Result<ImageType> {
        let mut ty: c_int = 0;
        check(unsafe { svbony_sys::SVBGetOutputImageType(self.id, &mut ty) })?;
        ImageType::try_from(ty)
    }

    /// Sets the output pixel format. Must be one of the formats in
    /// [`CameraProperty::supported_formats`].
    pub fn set_output_image_type(&self, ty: ImageType) -> Result<()> {
        check(unsafe { svbony_sys::SVBSetOutputImageType(self.id, ty as c_int) })
    }

    /// Returns the current region of interest and binning.
    pub fn roi(&self) -> Result<RoiFormat> {
        let (mut x, mut y, mut w, mut h, mut bin) = (0i32, 0i32, 0i32, 0i32, 0i32);
        check(unsafe {
            svbony_sys::SVBGetROIFormat(self.id, &mut x, &mut y, &mut w, &mut h, &mut bin)
        })?;
        Ok(RoiFormat {
            start_x: x,
            start_y: y,
            width: w,
            height: h,
            bin,
        })
    }

    /// Sets the ROI and binning. Capture must be stopped first.
    ///
    /// Width must be a multiple of 8, height a multiple of 2. Dimensions are
    /// post-binning.
    pub fn set_roi(&self, roi: &RoiFormat) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBSetROIFormat(
                self.id,
                roi.start_x,
                roi.start_y,
                roi.width,
                roi.height,
                roi.bin,
            )
        })
    }

    /// Returns the current ROI, binning, and binning mode.
    pub fn roi_ex(&self) -> Result<RoiFormatEx> {
        let (mut x, mut y, mut w, mut h, mut bin, mut mode) =
            (0i32, 0i32, 0i32, 0i32, 0i32, 0i32);
        check(unsafe {
            svbony_sys::SVBGetROIFormatEx(
                self.id, &mut x, &mut y, &mut w, &mut h, &mut bin, &mut mode,
            )
        })?;
        Ok(RoiFormatEx {
            start_x: x,
            start_y: y,
            width: w,
            height: h,
            bin,
            bin_mode: mode,
        })
    }

    /// Sets the ROI, binning, and binning mode (0 = average, 1 = sum).
    /// Capture must be stopped first.
    pub fn set_roi_ex(&self, roi: &RoiFormatEx) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBSetROIFormatEx(
                self.id,
                roi.start_x,
                roi.start_y,
                roi.width,
                roi.height,
                roi.bin,
                roi.bin_mode,
            )
        })
    }

    // --- Capture ---

    /// Starts continuous video capture.
    ///
    /// Frames are then retrieved with [`get_frame`](Self::get_frame) (or
    /// [`get_image`](Self::get_image) with the `image` feature).
    pub fn start_capture(&self) -> Result<()> {
        check(unsafe { svbony_sys::SVBStartVideoCapture(self.id) })
    }

    /// Stops video capture.
    pub fn stop_capture(&self) -> Result<()> {
        check(unsafe { svbony_sys::SVBStopVideoCapture(self.id) })
    }

    /// Reads one frame into `buf`.
    ///
    /// The buffer must be at least `width * height * bytes_per_pixel` bytes.
    /// `wait_ms` is the timeout in milliseconds (-1 = wait forever).
    /// A good default is `exposure_us / 1000 * 2 + 500`.
    pub fn get_frame(&self, buf: &mut [u8], wait_ms: i32) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBGetVideoData(
                self.id,
                buf.as_mut_ptr(),
                buf.len() as c_long,
                wait_ms as c_int,
            )
        })
    }

    /// Returns the number of frames dropped since capture started.
    ///
    /// Resets to 0 when capture is stopped.
    pub fn dropped_frames(&self) -> Result<i32> {
        let mut frames: c_int = 0;
        check(unsafe { svbony_sys::SVBGetDroppedFrames(self.id, &mut frames) })?;
        Ok(frames)
    }

    // --- Camera mode & trigger ---

    /// Returns the current camera mode (normal or trigger).
    pub fn mode(&self) -> Result<CameraMode> {
        let mut mode: c_int = 0;
        check(unsafe { svbony_sys::SVBGetCameraMode(self.id, &mut mode) })?;
        CameraMode::try_from(mode)
    }

    /// Sets the camera mode. Capture must be stopped first.
    pub fn set_mode(&self, mode: CameraMode) -> Result<()> {
        check(unsafe { svbony_sys::SVBSetCameraMode(self.id, mode as c_int) })
    }

    /// Sends a software trigger pulse.
    ///
    /// For edge triggers this starts a single exposure. For level triggers,
    /// call once to start and once to stop.
    pub fn send_soft_trigger(&self) -> Result<()> {
        check(unsafe { svbony_sys::SVBSendSoftTrigger(self.id) })
    }

    /// Configures a trigger output pin.
    ///
    /// `delay_us` and `duration_us` are in microseconds (0 to 2 000 000 000).
    /// Setting `duration_us <= 0` disables the pin.
    pub fn set_trigger_output(
        &self,
        pin: TrigOutputPin,
        high: bool,
        delay_us: i64,
        duration_us: i64,
    ) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBSetTriggerOutputIOConf(
                self.id,
                pin as c_int,
                if high { svbony_sys::SVB_TRUE } else { svbony_sys::SVB_FALSE },
                delay_us as c_long,
                duration_us as c_long,
            )
        })
    }

    /// Reads the current trigger output pin configuration.
    ///
    /// Returns `(active_high, delay_us, duration_us)`.
    pub fn get_trigger_output(&self, pin: TrigOutputPin) -> Result<(bool, i64, i64)> {
        let mut high: c_int = 0;
        let mut delay: c_long = 0;
        let mut duration: c_long = 0;
        check(unsafe {
            svbony_sys::SVBGetTriggerOutputIOConf(
                self.id,
                pin as c_int,
                &mut high,
                &mut delay,
                &mut duration,
            )
        })?;
        Ok((high != 0, delay as i64, duration as i64))
    }

    // --- Guide & misc ---

    /// Sends a pulse guide command (ST4) to move the telescope mount.
    ///
    /// `duration_ms` is the pulse duration in milliseconds.
    pub fn pulse_guide(&self, dir: GuideDirection, duration_ms: i32) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBPulseGuide(self.id, dir as c_int, duration_ms as c_int)
        })
    }

    /// Returns `true` if the camera supports pulse guiding (ST4).
    pub fn can_pulse_guide(&self) -> Result<bool> {
        let mut can: c_int = 0;
        check(unsafe { svbony_sys::SVBCanPulseGuide(self.id, &mut can) })?;
        Ok(can != 0)
    }

    /// Performs a one-shot automatic white balance.
    ///
    /// On success, read back [`ControlType::WbR`], [`ControlType::WbG`], and
    /// [`ControlType::WbB`] for the computed values.
    pub fn white_balance_once(&self) -> Result<()> {
        check(unsafe { svbony_sys::SVBWhiteBalanceOnce(self.id) })
    }

    /// Enables or disables automatic saving of camera parameters to a file.
    pub fn set_auto_save(&self, enable: bool) -> Result<()> {
        check(unsafe {
            svbony_sys::SVBSetAutoSaveParam(
                self.id,
                if enable { svbony_sys::SVB_TRUE } else { svbony_sys::SVB_FALSE },
            )
        })
    }

    /// Restores all camera parameters to factory defaults.
    pub fn restore_defaults(&self) -> Result<()> {
        check(unsafe { svbony_sys::SVBRestoreDefaultParam(self.id) })
    }

    /// Captures a frame and returns it as an [`image::DynamicImage`].
    ///
    /// This is a convenience method that queries the current ROI and output
    /// image type, allocates a buffer, captures one frame via
    /// [`get_frame`](Self::get_frame), and wraps the result.
    ///
    /// # Pixel format mapping
    ///
    /// | [`ImageType`] | [`DynamicImage`] variant |
    /// |---------------|--------------------------|
    /// | `Raw8`, `Y8` | `ImageLuma8` |
    /// | `Raw10`-`Raw16`, `Y10`-`Y16` | `ImageLuma16` (little-endian) |
    /// | `Rgb24` | `ImageRgb8` |
    /// | `Rgb32` | `ImageRgba8` |
    #[cfg(feature = "image")]
    pub fn get_image(&self, wait_ms: i32) -> Result<DynamicImage> {
        let roi = self.roi()?;
        let img_type = self.output_image_type()?;
        let w = roi.width as u32;
        let h = roi.height as u32;
        let buf_size = w as usize * h as usize * img_type.bytes_per_pixel();
        let mut buf = vec![0u8; buf_size];
        self.get_frame(&mut buf, wait_ms)?;

        let image = match img_type {
            ImageType::Raw8 | ImageType::Y8 => {
                DynamicImage::ImageLuma8(GrayImage::from_raw(w, h, buf).unwrap())
            }
            ImageType::Raw10
            | ImageType::Raw12
            | ImageType::Raw14
            | ImageType::Raw16
            | ImageType::Y10
            | ImageType::Y12
            | ImageType::Y14
            | ImageType::Y16 => {
                let pixels: Vec<u16> = buf
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                DynamicImage::ImageLuma16(
                    ImageBuffer::from_raw(w, h, pixels).unwrap(),
                )
            }
            ImageType::Rgb24 => {
                DynamicImage::ImageRgb8(
                    ImageBuffer::<Rgb<u8>, _>::from_raw(w, h, buf).unwrap(),
                )
            }
            ImageType::Rgb32 => {
                DynamicImage::ImageRgba8(
                    ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, buf).unwrap(),
                )
            }
        };
        Ok(image)
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            svbony_sys::SVBCloseCamera(self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_int;

    // --- Enum round-trip conversions ---

    #[test]
    fn image_type_round_trip() {
        for val in 0..=11 {
            let t = ImageType::try_from(val as c_int).unwrap();
            assert_eq!(t as c_int, val);
        }
        assert!(ImageType::try_from(-1 as c_int).is_err());
        assert!(ImageType::try_from(99 as c_int).is_err());
    }

    #[test]
    fn bayer_pattern_round_trip() {
        for val in 0..=3 {
            let b = BayerPattern::try_from(val as c_int).unwrap();
            assert_eq!(b as c_int, val);
        }
        assert!(BayerPattern::try_from(4).is_err());
    }

    #[test]
    fn control_type_round_trip() {
        for val in 0..=19 {
            let ct = ControlType::try_from(val as c_int).unwrap();
            assert_eq!(ct as c_int, val);
        }
        assert!(ControlType::try_from(20).is_err());
    }

    #[test]
    fn camera_mode_round_trip() {
        for val in 0..=6 {
            let m = CameraMode::try_from(val as c_int).unwrap();
            assert_eq!(m as c_int, val);
        }
        assert!(CameraMode::try_from(-1 as c_int).is_err());
        assert!(CameraMode::try_from(7).is_err());
    }

    #[test]
    fn flip_status_round_trip() {
        for val in 0..=3 {
            let f = FlipStatus::try_from(val as c_int).unwrap();
            assert_eq!(f as c_int, val);
        }
    }

    #[test]
    fn guide_direction_round_trip() {
        for val in 0..=3 {
            let g = GuideDirection::try_from(val as c_int).unwrap();
            assert_eq!(g as c_int, val);
        }
    }

    #[test]
    fn trig_output_pin_round_trip() {
        assert_eq!(TrigOutputPin::try_from(0).unwrap(), TrigOutputPin::PinA);
        assert_eq!(TrigOutputPin::try_from(1).unwrap(), TrigOutputPin::PinB);
        assert!(TrigOutputPin::try_from(2).is_err());
    }

    #[test]
    fn exposure_status_round_trip() {
        for val in 0..=3 {
            let e = ExposureStatus::try_from(val as c_int).unwrap();
            assert_eq!(e as c_int, val);
        }
    }

    // --- Error mapping ---

    #[test]
    fn check_success() {
        assert!(error::check(svbony_sys::SVB_SUCCESS).is_ok());
    }

    #[test]
    fn check_all_known_errors() {
        let cases: &[(c_int, Error)] = &[
            (1, Error::InvalidIndex),
            (2, Error::InvalidId),
            (3, Error::InvalidControlType),
            (4, Error::CameraClosed),
            (5, Error::CameraRemoved),
            (6, Error::InvalidPath),
            (7, Error::InvalidFileFormat),
            (8, Error::InvalidSize),
            (9, Error::InvalidImageType),
            (10, Error::OutOfBoundary),
            (11, Error::Timeout),
            (12, Error::InvalidSequence),
            (13, Error::BufferTooSmall),
            (14, Error::VideoModeActive),
            (15, Error::ExposureInProgress),
            (16, Error::GeneralError),
            (17, Error::InvalidMode),
            (18, Error::InvalidDirection),
            (19, Error::UnknownSensorType),
        ];
        for &(code, ref expected) in cases {
            assert_eq!(error::check(code).unwrap_err(), *expected);
        }
    }

    #[test]
    fn check_unknown_error() {
        assert_eq!(error::check(999).unwrap_err(), Error::Unknown(999));
    }

    #[test]
    fn error_display() {
        assert_eq!(Error::Timeout.to_string(), "timeout");
        assert_eq!(Error::Unknown(42).to_string(), "unknown error code: 42");
    }

    // --- Bytes per pixel ---

    #[test]
    fn bytes_per_pixel() {
        assert_eq!(ImageType::Raw8.bytes_per_pixel(), 1);
        assert_eq!(ImageType::Y8.bytes_per_pixel(), 1);
        assert_eq!(ImageType::Raw16.bytes_per_pixel(), 2);
        assert_eq!(ImageType::Raw10.bytes_per_pixel(), 2);
        assert_eq!(ImageType::Y16.bytes_per_pixel(), 2);
        assert_eq!(ImageType::Rgb24.bytes_per_pixel(), 3);
        assert_eq!(ImageType::Rgb32.bytes_per_pixel(), 4);
    }

    // --- Struct conversions from C types ---

    #[test]
    fn camera_info_from_c() {
        let mut c = unsafe { std::mem::zeroed::<svbony_sys::SVB_CAMERA_INFO>() };
        let name = b"TestCam\0";
        for (i, &b) in name.iter().enumerate() {
            c.FriendlyName[i] = b as i8;
        }
        let sn = b"SN123\0";
        for (i, &b) in sn.iter().enumerate() {
            c.CameraSN[i] = b as i8;
        }
        let port = b"USB3.0\0";
        for (i, &b) in port.iter().enumerate() {
            c.PortType[i] = b as i8;
        }
        c.DeviceID = 42;
        c.CameraID = 7;

        let info = CameraInfo::from(&c);
        assert_eq!(info.name, "TestCam");
        assert_eq!(info.serial, "SN123");
        assert_eq!(info.port_type, "USB3.0");
        assert_eq!(info.device_id, 42);
        assert_eq!(info.camera_id, 7);
    }

    #[test]
    fn camera_property_from_c() {
        let mut c = unsafe { std::mem::zeroed::<svbony_sys::SVB_CAMERA_PROPERTY>() };
        c.MaxWidth = 1920;
        c.MaxHeight = 1080;
        c.IsColorCam = 1;
        c.BayerPattern = svbony_sys::SVB_BAYER_GR;
        c.SupportedBins[0] = 1;
        c.SupportedBins[1] = 2;
        c.SupportedBins[2] = 0; // terminator
        c.SupportedVideoFormat[0] = svbony_sys::SVB_IMG_RAW8;
        c.SupportedVideoFormat[1] = svbony_sys::SVB_IMG_RAW16;
        c.SupportedVideoFormat[2] = svbony_sys::SVB_IMG_END; // terminator
        c.MaxBitDepth = 12;
        c.IsTriggerCam = 0;

        let prop = CameraProperty::from(&c);
        assert_eq!(prop.max_width, 1920);
        assert_eq!(prop.max_height, 1080);
        assert!(prop.is_color);
        assert_eq!(prop.bayer_pattern, BayerPattern::Gr);
        assert_eq!(prop.supported_bins, vec![1, 2]);
        assert_eq!(prop.supported_formats, vec![ImageType::Raw8, ImageType::Raw16]);
        assert_eq!(prop.max_bit_depth, 12);
        assert!(!prop.is_trigger_cam);
    }

    #[test]
    fn camera_property_ex_from_c() {
        let mut c = unsafe { std::mem::zeroed::<svbony_sys::SVB_CAMERA_PROPERTY_EX>() };
        c.bSupportPulseGuide = 1;
        c.bSupportControlTemp = 0;

        let prop = CameraPropertyEx::from(&c);
        assert!(prop.supports_pulse_guide);
        assert!(!prop.supports_temp_control);
    }

    #[test]
    fn control_caps_from_c() {
        let mut c = unsafe { std::mem::zeroed::<svbony_sys::SVB_CONTROL_CAPS>() };
        let name = b"Gain\0";
        for (i, &b) in name.iter().enumerate() {
            c.Name[i] = b as i8;
        }
        let desc = b"Camera gain\0";
        for (i, &b) in desc.iter().enumerate() {
            c.Description[i] = b as i8;
        }
        c.MaxValue = 300;
        c.MinValue = 0;
        c.DefaultValue = 10;
        c.IsAutoSupported = 1;
        c.IsWritable = 1;
        c.ControlType = svbony_sys::SVB_GAIN;

        let caps = ControlCaps::from(&c);
        assert_eq!(caps.name, "Gain");
        assert_eq!(caps.description, "Camera gain");
        assert_eq!(caps.max_value, 300);
        assert_eq!(caps.min_value, 0);
        assert_eq!(caps.default_value, 10);
        assert!(caps.is_auto_supported);
        assert!(caps.is_writable);
        assert_eq!(caps.control_type, ControlType::Gain);
    }

    // --- RoiFormat ---

    #[test]
    fn roi_format_fields() {
        let roi = RoiFormat {
            start_x: 0,
            start_y: 0,
            width: 640,
            height: 480,
            bin: 2,
        };
        assert_eq!(roi.width, 640);
        assert_eq!(roi.bin, 2);
    }

    // --- Image feature tests ---

    #[cfg(feature = "image")]
    mod image_tests {
        use image::DynamicImage;

        fn make_luma8(w: u32, h: u32, val: u8) -> Vec<u8> {
            vec![val; w as usize * h as usize]
        }

        fn make_luma16_le(w: u32, h: u32, val: u16) -> Vec<u8> {
            let bytes = val.to_le_bytes();
            let mut buf = Vec::with_capacity(w as usize * h as usize * 2);
            for _ in 0..(w as usize * h as usize) {
                buf.extend_from_slice(&bytes);
            }
            buf
        }

        fn make_rgb24(w: u32, h: u32) -> Vec<u8> {
            let mut buf = Vec::with_capacity(w as usize * h as usize * 3);
            for _ in 0..(w as usize * h as usize) {
                buf.extend_from_slice(&[255, 0, 128]);
            }
            buf
        }

        fn make_rgba32(w: u32, h: u32) -> Vec<u8> {
            let mut buf = Vec::with_capacity(w as usize * h as usize * 4);
            for _ in 0..(w as usize * h as usize) {
                buf.extend_from_slice(&[255, 0, 128, 200]);
            }
            buf
        }

        #[test]
        fn buf_to_luma8() {
            let (w, h) = (4u32, 3u32);
            let buf = make_luma8(w, h, 42);
            let img = DynamicImage::ImageLuma8(
                image::GrayImage::from_raw(w, h, buf).unwrap(),
            );
            assert_eq!(img.width(), w);
            assert_eq!(img.height(), h);
            assert_eq!(img.as_luma8().unwrap()[(0, 0)].0[0], 42);
        }

        #[test]
        fn buf_to_luma16() {
            let (w, h) = (4u32, 3u32);
            let buf = make_luma16_le(w, h, 1000);
            let pixels: Vec<u16> = buf
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let img = DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(w, h, pixels).unwrap(),
            );
            assert_eq!(img.width(), w);
            assert_eq!(img.height(), h);
            assert_eq!(img.as_luma16().unwrap()[(0, 0)].0[0], 1000);
        }

        #[test]
        fn buf_to_rgb24() {
            let (w, h) = (4u32, 3u32);
            let buf = make_rgb24(w, h);
            let img = DynamicImage::ImageRgb8(
                image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(w, h, buf).unwrap(),
            );
            assert_eq!(img.as_rgb8().unwrap()[(0, 0)].0, [255, 0, 128]);
        }

        #[test]
        fn buf_to_rgba32() {
            let (w, h) = (4u32, 3u32);
            let buf = make_rgba32(w, h);
            let img = DynamicImage::ImageRgba8(
                image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(w, h, buf).unwrap(),
            );
            assert_eq!(img.as_rgba8().unwrap()[(0, 0)].0, [255, 0, 128, 200]);
        }
    }
}
