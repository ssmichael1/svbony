use crate::lowlevel as ll;

use camera::{Camera, CameraError, CameraFrame, FrameCallback};
use numeris::image::Image;

pub use ll::SVBCameraInfo;
pub use ll::{PixelType, SVBControlCaps, SVBControlType};
pub use ll::{SVBBayerPattern, SVBCameraProperty, SVBError, SVBImageType};

use std::sync::{Arc, Mutex};

type SVBResult<T> = Result<T, SVBError>;

pub enum FrameData<'a> {
    U8Frame(&'a [u8]),
    U16Frame(&'a [u16]),
}

#[derive(Clone)]
pub struct SVBonyCamera {
    id: i32,
    info: SVBCameraInfo,
    property: SVBCameraProperty,
    pixel_pitch: f64,
    capabilities: Vec<SVBControlCaps>,
    running: Arc<Mutex<bool>>,
    callback: Arc<Mutex<Option<Box<FrameCallback>>>>,
    exposure: Arc<Mutex<f64>>,
}

/// Get a list of SVBony cameras connected to the host system
///
/// # Returns
///   A vector of SVBCameraInfo struct describing the connected cameras
pub fn get_connected_cameras() -> SVBResult<Vec<SVBCameraInfo>> {
    let ncam = ll::get_number_of_connected_camera();
    (0..ncam).map(ll::get_camera_info).collect()
}

impl Camera for SVBonyCamera {
    fn name(&self) -> String {
        self.info.friendly_name.clone()
    }

    fn connect(&mut self) -> Result<(), CameraError> {
        ll::open_camera(self.id).map_err(|e| match e {
            SVBError::InvalidId => CameraError::Connection,
            _ => CameraError::Other(format!("Failed to open camera: {}", e)),
        })
    }

    fn disconnect(&mut self) -> Result<(), CameraError> {
        ll::close_camera(&self.id).map_err(|e| match e {
            SVBError::InvalidId => CameraError::Connection,
            _ => CameraError::Other(format!("Failed to close camera: {}", e)),
        })
    }

    /// Set exposure time in seconds
    fn set_exposure(&mut self, exposure: f64) -> Result<(), CameraError> {
        let exposure_us = (exposure * 1_000_000.0) as i32;
        self.set_control_value(SVBControlType::SVBExposure, exposure_us)
            .map_err(|e| CameraError::Other(format!("Failed to set exposure: {}", e)))?;
        // Now get the exposure
        let exp = self
            .get_control_value(SVBControlType::SVBExposure)
            .map_err(|e| CameraError::Other(format!("Failed to get exposure: {}", e)))?;
        *self.exposure.lock().unwrap() = exp as f64 / 1_000_000.0;
        Ok(())
    }

    fn get_exposure(&self) -> Result<f64, CameraError> {
        Ok(*self.exposure.lock().unwrap())
    }

    fn get_exposure_limits(&self) -> Result<(f64, f64), CameraError> {
        if let Some(cap) = self.get_control_info(SVBControlType::SVBExposure) {
            Ok((
                cap.min_value as f64 / 1_000_000.0,
                cap.max_value as f64 / 1_000_000.0,
            ))
        } else {
            Err(CameraError::NotSupported)
        }
    }

    fn set_gain(&mut self, gain: f64) -> Result<(), CameraError> {
        let gain_val = gain as i32;
        self.set_control_value(SVBControlType::SVBGain, gain_val)
            .map_err(|e| CameraError::Other(format!("Failed to set gain: {}", e)))
    }

    fn get_gain(&self) -> Result<f64, CameraError> {
        self.get_control_value(SVBControlType::SVBGain)
            .map(|v| v as f64)
            .map_err(|e| CameraError::Other(format!("Failed to get gain: {}", e)))
    }

    fn set_frame_callback<F>(&mut self, cb: F) -> Result<(), CameraError>
    where
        F: Fn(&CameraFrame) -> Result<(), CameraError> + Send + Sync + 'static,
    {
        *self.callback.lock().unwrap() = Some(Box::new(cb));
        Ok(())
    }

    fn start(&mut self) -> Result<(), CameraError> {
        let mut cam = self.clone();
        std::thread::spawn(move || {
            cam.run().unwrap();
        });
        Ok(())
    }

    fn stop(&mut self) -> Result<(), CameraError> {
        SVBonyCamera::stop(self).map_err(|e| match e {
            SVBError::InvalidId => CameraError::Connection,
            _ => CameraError::Other(format!("Failed to stop camera: {}", e)),
        })?;
        Ok(())
    }
}

impl SVBonyCamera {
    /// Create a new camera object
    ///
    /// # Arguments
    /// * `num` - The camera number, index of the camera in the list of connected cameras from `get_connected_cameras`
    ///
    /// # Returns
    /// The camera object if successful
    /// Error if the camera cannot be opened
    ///
    pub fn new(num: usize) -> SVBResult<SVBonyCamera> {
        let info = ll::get_camera_info(num)?;
        let id: i32 = info.camera_id;
        ll::open_camera(id)?;

        Ok(SVBonyCamera {
            id: info.camera_id,
            info,
            property: ll::get_camera_property(&id)?,
            pixel_pitch: ll::get_pixel_size_microns(&id)? as f64,
            capabilities: {
                (0..ll::get_number_of_controls(&id)?)
                    .map(|i| -> SVBResult<SVBControlCaps> { ll::get_control_info(&id, i) })
                    .collect::<SVBResult<Vec<SVBControlCaps>>>()?
            },
            running: Arc::new(Mutex::new(false)),
            callback: Arc::new(Mutex::new(None)),
            exposure: Arc::new(Mutex::new({
                ll::get_control_value(&id, SVBControlType::SVBExposure)
                    .map(|v| v.0)
                    .unwrap_or(0) as f64
                    / 1_000_000.0
            })),
        })
    }

    pub fn run(&mut self) -> SVBResult<()> {
        let nbuffers = 10;
        let mut buffers: Vec<Vec<u16>> = (0..nbuffers)
            .map(|_| vec![0u16; (self.max_width() * self.max_height()) as usize])
            .collect();
        let bufcounter: usize = 0;

        self.start_video_capture().unwrap();
        *self.running.lock().unwrap() = true;

        while *self.running.lock().unwrap() {
            let exposure = *self.exposure.lock().unwrap();
            let wait_ms = (exposure * 1_000_000.0) * 2.0 / 1000.0 + 500.0;

            // Recommended timeout from vendor is 2x exposure time + 500ms
            let ts = match self.get_frame(&mut buffers[bufcounter], wait_ms as i32) {
                Ok(ts) => ts,
                Err(e) => {
                    if e == SVBError::Timeout && !(*self.running.lock().unwrap()) {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            };
            if let Some(ref cb) = *self.callback.lock().unwrap() {
                cb(&CameraFrame {
                    exposure,
                    center_of_integration: ts,
                    bit_depth: None,
                    frame: Image::<u16>::from_data(
                        self.max_width() as usize,
                        self.max_height() as usize,
                        buffers[bufcounter].clone(),
                    )
                    .unwrap()
                    .into(),
                })
                .unwrap();
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> SVBResult<()> {
        println!("setting running to false");
        *self.running.lock().unwrap() = false;
        self.stop_video_capture().unwrap();
        Ok(())
    }

    /// Close the camera
    ///
    /// # Notes:
    ///     The camera will be automatically closed when the object is dropped
    ///
    /// # Returns
    ///   Empty result if successful
    ///  Error if the camera cannot be closed
    ///
    pub fn close_camera(&mut self) -> SVBResult<()> {
        ll::close_camera(&self.id)
    }

    /// Get the info structure describign the camera
    ///
    /// # Returns
    ///    The camera info in SVBCameraInfo struct
    pub fn get_info(&self) -> &SVBCameraInfo {
        &self.info
    }

    /// Get number of dropped frames
    /// # Returns
    ///   The number of dropped frames
    pub fn dropped_frames(&self) -> i32 {
        ll::get_dropped_frames(&self.id).unwrap_or(0)
    }

    /// Get the camera pixel pitch in microns
    ///
    /// # Returns
    ///    The pixel pitch in microns
    pub fn pixel_pitch(&self) -> f64 {
        self.pixel_pitch
    }
    /// Get the camera properties
    ///
    /// # Returns
    ///    The camera properties in SVBCameraProperty struct
    pub fn get_properties(&self) -> &ll::SVBCameraProperty {
        &self.property
    }

    /// Get maximum width (columns) in pixels of the camera
    ///
    /// # Returns
    ///   The maximum width in pixels
    pub fn max_width(&self) -> i32 {
        self.property.max_width
    }

    /// Get the maximum height (rows) in pixels of the camera
    ///
    /// # Returns
    ///   The maximum height in pixels
    pub fn max_height(&self) -> i32 {
        self.property.max_height
    }

    /// Query color or monochrome camera
    ///
    /// # Returns
    ///     True if the camera is color, false if monochrome
    pub fn is_color_cam(&self) -> bool {
        self.property.is_color_cam
    }

    /// Get the Bayer pattern of the camera
    ///
    /// # Notes
    ///      Only valid for color cameras
    ///
    /// # Returns
    ///     The Bayer pattern as SVBBayerPattern enum
    pub fn bayer_pattern(&self) -> SVBBayerPattern {
        self.property.bayer_pattern.clone()
    }

    /// Get supported pixel binning
    ///
    /// # Notes
    ///      Binning of pixels is a method of combining the charge from adjacent pixels
    ///
    /// # Returns
    ///    A vector of supported pixel binning values
    pub fn supported_bins(&self) -> Vec<i32> {
        self.property.supported_bins.clone()
    }

    /// Get supported video formats
    ///
    /// # Returns
    ///   A vector of supported video formats as SVBImageType enum
    pub fn supported_video_formats(&self) -> Vec<SVBImageType> {
        self.property.supported_video_format.clone()
    }

    /// Get info for type of camera control
    ///
    /// # Arguments
    ///   * `ctrl` - The control type
    ///
    /// # Returns
    ///  The control info in SVBControlCaps struct
    pub fn get_control_info(&self, ctrl: SVBControlType) -> Option<SVBControlCaps> {
        self.capabilities
            .iter()
            .find(|&x| x.control_type == ctrl)
            .cloned()
    }

    /// Set the value of a camera control
    ///
    /// # Arguments
    ///  * `ctrl` - The control type
    ///  * `value` - The value to set
    ///
    /// # Returns
    ///  Empty result if successful
    /// Error if the value is invalid
    ///
    pub fn set_control_value(&self, ctrl: SVBControlType, value: i32) -> SVBResult<()> {
        ll::set_control_value(&self.id, ctrl, value, false)
    }

    /// Query the value of a camera control
    ///
    /// # Arguments
    /// * `ctrl` - The control type
    ///
    /// # Returns
    /// The value of the control
    ///
    /// # Errors
    /// Error if the control is not found
    pub fn get_control_value(&self, ctrl: SVBControlType) -> SVBResult<i32> {
        ll::get_control_value(&self.id, ctrl).map(|v| v.0)
    }

    /// Start video capture
    ///
    /// # Note: User must call get_video_frame to get the video frames
    ///
    /// # Returns
    /// Empty result if successful
    /// Error if the video capture fails
    pub fn start_video_capture(&self) -> SVBResult<()> {
        ll::start_capture(&self.id)
    }

    pub fn get_frame<T>(
        &self,
        data: &mut [T],
        wait_ms: i32,
    ) -> Result<chrono::DateTime<chrono::Utc>, SVBError>
    where
        T: ll::PixelType,
    {
        ll::get_video_data(&self.id, data, wait_ms)
    }

    /// Stop video capture
    ///
    /// # Returns
    /// Empty result if successful
    /// Error if the stopping the video capture fails
    pub fn stop_video_capture(&self) -> SVBResult<()> {
        ll::stop_capture(&self.id)
    }
}

impl Drop for SVBonyCamera {
    fn drop(&mut self) {
        if self.id != -1 {
            let _ = ll::close_camera(&self.id);
        }
    }
}

impl std::fmt::Display for SVBonyCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "SVBonyCamera")?;
        writeln!(f, "    Name: {}", self.info.friendly_name)?;
        writeln!(f, "    Serial Number: {}", self.info.serial_number)?;
        writeln!(f, "    Port Type: {}", self.info.port_type)?;
        writeln!(
            f,
            "    Format: {} x {}",
            self.max_width(),
            self.max_height()
        )?;
        writeln!(f, "    Color: {}", self.is_color_cam())?;
        if self.is_color_cam() {
            writeln!(f, "    Bayer Pattern: {:?}", self.bayer_pattern())?;
        }
        writeln!(f, "    Supported Bins: {:?}", self.supported_bins())?;
        writeln!(
            f,
            "    Supported Video Formats: {:?}",
            self.supported_video_formats()
        )?;
        writeln!(f, "    Pixel Pitch: {:.2} µm", self.pixel_pitch)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_connected_cameras() {
        let cameras = get_connected_cameras().unwrap();
        println!("cameras = {:?}", cameras);
    }

    #[test]
    fn test_video_capture() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            return;
        }
        let cam = SVBonyCamera::new(0).unwrap();
        cam.start_video_capture().unwrap();
        let mut data = vec![0u8; (cam.max_width() * cam.max_height()) as usize];
        let ts = cam.get_frame(&mut data, 1000).unwrap();
        println!("ts = {}", ts);
        cam.stop_video_capture().unwrap();
    }

    #[test]
    fn test_capabilities() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            return;
        }
        let cam = SVBonyCamera::new(0).unwrap();
        println!("cam = {}", cam);
        cam.capabilities.iter().for_each(|c| {
            println!("{}", c);
        });
    }

    #[test]
    fn test_in_thread() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            println!("No cameras found");
            return;
        }
        let mut cam = SVBonyCamera::new(0).unwrap();
        cam.set_exposure(0.2).unwrap();

        cam.set_frame_callback(|frame: &CameraFrame| -> Result<(), CameraError> {
            println!(
                "frame: exp={} ts={} size={}x{}",
                frame.exposure,
                frame.center_of_integration,
                frame.frame.width(),
                frame.frame.height()
            );
            Ok(())
        })
        .unwrap();

        let mut camclone = cam.clone();
        let handle = std::thread::spawn(move || -> SVBResult<()> { camclone.run() });
        std::thread::sleep(std::time::Duration::from_secs(3));
        cam.stop().unwrap();
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn test_camera() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            return;
        }
        let mut cam = SVBonyCamera::new(0).unwrap();

        cam.set_gain(30.0).unwrap();

        println!("cam = {}", cam);
        println!("exposure = {}", cam.get_exposure().unwrap());
        println!("gain = {}", cam.get_gain().unwrap());
    }
}
