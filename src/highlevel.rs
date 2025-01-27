use crate::lowlevel as ll;

pub use ll::SVBCameraInfo;
pub use ll::{PixelType, SVBControlCaps, SVBControlType};
pub use ll::{SVBBayerPattern, SVBCameraProperty, SVBErrorCode, SVBImageType};

use std::sync::{Arc, Mutex};

pub enum FrameData<'a> {
    U8Frame(&'a [u8]),
    U16Frame(&'a [u16]),
}

pub trait CameraCallback: Send + Sync {
    fn on_video_frame(&mut self, data: &FrameData, timestamp: chrono::DateTime<chrono::Utc>);
}

pub type CameraCallbackFunction = dyn Fn(&FrameData, chrono::DateTime<chrono::Utc>) + Send + Sync;

#[derive(Clone)]
pub struct SVBonyCamera {
    id: i32,
    info: SVBCameraInfo,
    property: SVBCameraProperty,
    pixel_pitch: f64,
    capabilities: Vec<SVBControlCaps>,
    callbacks: Vec<Arc<Mutex<dyn CameraCallback>>>,
    function_callbacks: Vec<Arc<CameraCallbackFunction>>,
    running: Arc<Mutex<bool>>,
}

type SVBResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Get a list of SVBony cameras connected to the host system
///
/// # Returns
///   A vector of SVBCameraInfo struct describing the connected cameras
pub fn get_connected_cameras() -> SVBResult<Vec<SVBCameraInfo>> {
    let ncam = ll::get_number_of_connected_camera();
    (0..ncam)
        .map(|i| match ll::get_camera_info(i) {
            Ok(info) => Ok(info),
            Err(e) => Err(e.into()),
        })
        .collect()
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
                    .map(|i| -> SVBResult<SVBControlCaps> {
                        match ll::get_control_info(&id, i) {
                            Ok(info) => Ok(info),
                            Err(e) => Err(e.into()),
                        }
                    })
                    .collect::<SVBResult<Vec<SVBControlCaps>>>()?
            },
            callbacks: Vec::new(),
            function_callbacks: Vec::new(),
            running: Arc::new(Mutex::new(false)),
        })
    }

    pub fn run(&mut self) -> SVBResult<()> {
        let nbuffers = 10;
        let mut buffers: Vec<Vec<u16>> = (0..nbuffers)
            .map(|_| vec![0u16; (self.max_width() * self.max_height()) as usize])
            .collect();
        let mut bufcounter = 0;

        // Recommended timeout from vendor is 2x exposure time + 500ms
        // and exposure is queried in microseconds
        let wait_ms = self.exposure()? * 2 / 1000 + 500;

        self.start_video_capture().unwrap();
        *self.running.lock().unwrap() = true;

        while *self.running.lock().unwrap() {
            let ts = match self.get_frame(&mut buffers[bufcounter], wait_ms) {
                Ok(ts) => ts,
                Err(e) => {
                    if e == SVBErrorCode::SVBErrorTimeout && !(*self.running.lock().unwrap()) {
                        break;
                    } else {
                        return Err(e.into());
                    }
                }
            };
            for cb in self.callbacks.iter_mut() {
                cb.lock()
                    .unwrap()
                    .on_video_frame(&FrameData::U16Frame(&buffers[bufcounter]), ts);
            }
            for cb in self.function_callbacks.iter() {
                cb(&FrameData::U16Frame(&buffers[bufcounter]), ts);
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        println!("setting running to false");
        *self.running.lock().unwrap() = false;
        self.stop_video_capture().unwrap();
    }

    /// Add a callback to the camera
    /// # Arguments
    /// * `callback` - The callback object
    ///
    /// # Notes
    ///    The callback object must implement the CameraCallback trait
    ///
    /// # Returns
    ///   Empty result if successful
    ///  Error if the callback cannot be added
    pub fn add_callback<F>(&mut self, callback: F) -> SVBResult<()>
    where
        F: CameraCallback + Send + Sync + 'static,
    {
        self.callbacks.push(Arc::new(Mutex::new(callback)));
        Ok(())
    }

    pub fn add_function_callback<F>(&mut self, callback: F) -> SVBResult<()>
    where
        F: Fn(&FrameData, chrono::DateTime<chrono::Utc>) + Send + Sync + 'static,
    {
        self.function_callbacks.push(Arc::new(callback));
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
        match ll::close_camera(&self.id) {
            Ok(_) => {
                self.id = -1;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
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

    /// Get the camera exposure time in microseconds
    ///
    /// # Returns
    ///     The exposure time in microseconds
    pub fn exposure(&self) -> SVBResult<i32> {
        self.get_control_value(SVBControlType::SVBExposure)
    }

    /// Get the camera pixel pitch in microns
    ///
    /// # Returns
    ///    The pixel pitch in microns
    pub fn pixel_pitch(&self) -> f64 {
        self.pixel_pitch
    }

    /// Set the camera exposure time in microseconds
    ///
    /// # Arguments
    ///     * `value` - The exposure time in microseconds
    ///
    /// # Returns
    ///    Empty result if successful
    ///   Error if the exposure time is invalid
    pub fn set_exposure(&self, value: i32) -> SVBResult<()> {
        self.set_control_value(SVBControlType::SVBExposure, value)
    }

    /// Get the camera gain
    ///
    /// # Returns
    ///     The gain value
    ///
    pub fn gain(&self) -> SVBResult<i32> {
        self.get_control_value(SVBControlType::SVBGain)
    }

    /// Set the camera gain
    /// # Arguments
    ///    * `value` - The gain value
    ///
    /// # Returns
    ///    Empty result if successful
    ///    Error if the gain value is invalid
    ///    
    pub fn set_gain(&self, value: i32) -> SVBResult<()> {
        self.set_control_value(SVBControlType::SVBGain, value)
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
    pub fn get_control_info(&self, ctrl: SVBControlType) -> SVBResult<SVBControlCaps> {
        self.capabilities
            .iter()
            .find(|&x| x.control_type == ctrl)
            .cloned()
            .ok_or_else(|| "Control not found".into())
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
        match ll::set_control_value(&self.id, ctrl, value, false) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
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
        match ll::get_control_value(&self.id, ctrl) {
            Ok(value) => Ok(value.0),
            Err(e) => Err(e.into()),
        }
    }

    /// Start video capture
    ///
    /// # Note: User must call get_video_frame to get the video frames
    ///
    /// # Returns
    /// Empty result if successful
    /// Error if the video capture fails
    pub fn start_video_capture(&self) -> SVBResult<()> {
        match ll::start_capture(&self.id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_frame<T>(
        &self,
        data: &mut [T],
        wait_ms: i32,
    ) -> Result<chrono::DateTime<chrono::Utc>, SVBErrorCode>
    where
        T: ll::PixelType,
    {
        match ll::get_video_data(&self.id, data, wait_ms) {
            Ok(ts) => Ok(ts),
            Err(e) => Err(e),
        }
    }

    /// Stop video capture
    ///
    /// # Returns
    /// Empty result if successful
    /// Error if the stopping the video capture fails
    pub fn stop_video_capture(&self) -> SVBResult<()> {
        match ll::stop_capture(&self.id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
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
    use std::io::Write;

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
    fn test_write() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            return;
        }
        let mut cam = SVBonyCamera::new(0).unwrap();
        cam.set_exposure(10000).unwrap();
        cam.set_control_value(SVBControlType::SVBFrameSpeedMode, 0)
            .unwrap();
        println!("cam = {}", cam);

        #[derive(Debug)]
        struct RawWriteCallback {
            fs: std::fs::File,
        }
        impl RawWriteCallback {
            fn new() -> Self {
                let fs = std::fs::File::create("test.raw").unwrap();
                RawWriteCallback { fs }
            }
        }

        impl CameraCallback for RawWriteCallback {
            fn on_video_frame(
                &mut self,
                data: &FrameData,
                _timestamp: chrono::DateTime<chrono::Utc>,
            ) {
                match data {
                    FrameData::U8Frame(data) => {
                        self.fs.write_all(data).unwrap();
                    }
                    FrameData::U16Frame(data) => {
                        self.fs
                            .write_all(unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const u8,
                                    std::mem::size_of_val(*data),
                                )
                            })
                            .unwrap();
                    }
                }
            }
        }
        let cc = RawWriteCallback::new();
        cam.add_callback(cc).unwrap();
        let mut camclone = cam.clone();
        let handle = std::thread::spawn(move || {
            camclone.run().unwrap();
        });
        std::thread::sleep(std::time::Duration::from_secs(5));
        cam.stop();
        handle.join().unwrap();
    }

    #[test]
    fn test_in_thread() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            return;
        }
        let mut cam = SVBonyCamera::new(0).unwrap();
        cam.set_exposure(100000).unwrap();
        cam.add_function_callback(|_data, ts| {
            println!("ts = {}", ts);
        })
        .unwrap();
        let mut camclone = cam.clone();
        let handle = std::thread::spawn(move || -> SVBResult<()> { camclone.run() });
        std::thread::sleep(std::time::Duration::from_secs(1));
        cam.stop();
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn test_camera() {
        let cameras = get_connected_cameras().unwrap();
        if cameras.is_empty() {
            return;
        }
        let cam = SVBonyCamera::new(0).unwrap();

        cam.set_gain(30).unwrap();

        println!("cam = {}", cam);
        println!("exposure = {}", cam.exposure().unwrap());
        println!("gain = {}", cam.gain().unwrap());
    }
}
