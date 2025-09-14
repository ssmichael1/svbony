use super::structs::*;
use super::types::*;
use super::SVBError;
use std::ffi::{c_char, c_float, c_int, c_long, c_uchar, c_uint};

//#[link(name = "SVBCameraSDK")]
extern "C" {
    fn SVBGetNumOfConnectedCameras() -> c_int;
    fn SVBGetCameraInfo(info: *mut LLSVBCameraInfo, id: c_int) -> c_int;
    fn SVBGetCameraProperty(id: c_int, property: *mut LLSVBCameraProperty) -> c_int;
    fn SVBGetCameraPropertyEx(id: c_int, property: *mut LLSVBCameraPropertyEx) -> c_int;
    fn SVBOpenCamera(id: c_int) -> c_int;
    fn SVBCloseCamera(id: c_int) -> c_int;
    fn SVBGetNumOfControls(id: c_int, num: *mut i32) -> c_int;
    fn SVBGetControlCaps(id: c_int, control_id: c_int, caps: *mut LLSVBControlCaps) -> c_int;
    fn SVBGetControlValue(
        id: c_int,
        control_id: c_int,
        value: *mut c_long,
        auto: *mut i32,
    ) -> c_int;
    fn SVBSetControlValue(id: c_int, ctrl: c_uint, value: c_long, auto: i32) -> c_int;
    fn SVBGetOutputImageType(id: c_int, image_type: *mut i32) -> c_int;
    fn SVGSetOutputImageType(id: c_int, image_type: i32) -> c_int;
    fn SVBSetROIFormat(
        id: c_int,
        startx: c_int,
        starty: c_int,
        width: c_int,
        height: c_int,
        bin: c_int,
    ) -> c_int;
    fn SVBSetROIFormatEx(
        id: c_int,
        startx: c_int,
        starty: c_int,
        width: c_int,
        height: c_int,
        bin: c_int,
        mode: c_int,
    ) -> c_int;
    fn SVBGetROIFormat(
        id: c_int,
        startx: *mut c_int,
        starty: *mut c_int,
        width: *mut c_int,
        height: *mut c_int,
        bin: *mut c_int,
    ) -> c_int;
    fn SVBGetROIFormatEx(
        id: c_int,
        startx: *mut c_int,
        starty: *mut c_int,
        width: *mut c_int,
        height: *mut c_int,
        bin: *mut c_int,
        mode: *mut c_int,
    ) -> c_int;
    fn SVBGetDroppedFrames(id: c_int, dropped_frames: *mut c_int) -> c_int;
    fn SVBStartVideoCapture(id: c_int) -> c_int;
    fn SVBStopVideoCapture(id: c_int) -> c_int;
    fn SVBGetVideoData(id: c_int, data: *mut c_uchar, size: c_long, waitms: c_int) -> c_int;
    fn SVBWhiteBalanceOnce(id: c_int) -> c_int;
    fn SVBGetCameraMode(id: c_int, mode: *mut c_int) -> c_int;
    fn SVGSetCameraMode(id: c_int, mode: c_int) -> c_int;
    fn SVBSendSoftTrigger(id: c_int) -> c_int;
    fn SVBGetSensorPixelSize(id: c_int, pixel_size: *mut c_float) -> c_int;
    fn SVBRestoreDefaultParam(id: c_int) -> c_int;
    fn SVBSetAutoSaveParam(id: c_int, save: c_int) -> c_int;
    fn SVBGetSDKVersion() -> *const c_char;
}

pub fn restore_default_parameters(id: &i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBRestoreDefaultParam(*id) }.into();
    result.into()
}

pub fn get_sdk_version() -> String {
    let version = unsafe { SVBGetSDKVersion() };
    let version = unsafe { std::ffi::CStr::from_ptr(version) };
    version.to_string_lossy().to_string()
}

pub fn set_auto_save(id: &i32, save: bool) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBSetAutoSaveParam(*id, if save { 1 } else { 0 }) }.into();
    result.into()
}

pub fn open_camera(id: i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBOpenCamera(id) }.into();
    result.into()
}

pub fn close_camera(id: &i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBCloseCamera(*id) }.into();
    result.into()
}

pub fn get_number_of_connected_camera() -> usize {
    unsafe { SVBGetNumOfConnectedCameras() as usize }
}

pub fn get_number_of_controls(id: &i32) -> Result<usize, SVBError> {
    let mut num: i32 = 0;
    let result = unsafe { SVBGetNumOfControls(*id, &mut num) }.into();
    if result == SVBError::Success {
        Ok(num as usize)
    } else {
        Err(result)
    }
}

pub fn get_output_image_type(id: i32) -> Result<SVBImageType, SVBError> {
    let mut image_type = 0;
    let result = unsafe { SVBGetOutputImageType(id, &mut image_type) }.into();
    if result == SVBError::Success {
        Ok(image_type.into())
    } else {
        Err(result)
    }
}

pub fn set_output_image_type(id: i32, image_type: SVBImageType) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVGSetOutputImageType(id, image_type as i32) }.into();
    result.into()
}

pub fn get_control_info(id: &i32, control_id: usize) -> Result<SVBControlCaps, SVBError> {
    let mut caps = LLSVBControlCaps {
        name: [0; 64],
        description: [0; 128],
        max_value: 0,
        min_value: 0,
        default_value: 0,
        is_auto_supported: 0,
        is_writeable: 0,
        control_type: 0,
        unused: [0; 32],
    };
    let result = unsafe { SVBGetControlCaps(*id, control_id as i32, &mut caps) }.into();

    if result == SVBError::Success {
        Ok(caps.into())
    } else {
        Err(result)
    }
}

pub fn get_control_value(id: &i32, ctrl: SVBControlType) -> Result<(i32, bool), SVBError> {
    let mut value = 0 as c_long;
    let mut auto = 0;
    let result = unsafe { SVBGetControlValue(*id, ctrl as i32, &mut value, &mut auto) }.into();

    if result == SVBError::Success {
        Ok((value as i32, auto != 0))
    } else {
        Err(result)
    }
}

pub fn get_camera_mode(id: &i32) -> Result<SVBCameraMode, SVBError> {
    let mut mode = 0;
    let result = unsafe { SVBGetCameraMode(*id, &mut mode) }.into();
    if result == SVBError::Success {
        Ok(mode.into())
    } else {
        Err(result)
    }
}

pub fn set_camera_mode(id: &i32, mode: SVBCameraMode) -> Result<(), SVBError> {
    let result = unsafe { SVGSetCameraMode(*id, mode as i32) }.into();
    if result == SVBError::Success {
        Ok(())
    } else {
        Err(result)
    }
}

pub fn get_pixel_size_microns(id: &i32) -> Result<f32, SVBError> {
    let mut pixel_size = 0.0 as c_float;
    let result = unsafe { SVBGetSensorPixelSize(*id, &mut pixel_size) }.into();
    if result == SVBError::Success {
        Ok(pixel_size as f32)
    } else {
        Err(result)
    }
}

pub fn set_control_value(
    id: &i32,
    ctrl: SVBControlType,
    value: i32,
    auto: bool,
) -> Result<(), SVBError> {
    let result: SVBError = unsafe {
        SVBSetControlValue(
            *id,
            ctrl as c_uint,
            value as c_long,
            if auto { 1 } else { 0 },
        )
    }
    .into();
    result.into()
}

pub fn get_camera_property(id: &i32) -> Result<SVBCameraProperty, SVBError> {
    let mut property = LLSVBCameraProperty {
        max_height: 0,
        max_width: 0,
        is_color_cam: 0,
        bayer_pattern: 0,
        supported_bins: [0; 16],
        supported_video_format: [0; 8],
        max_bit_depth: 0,
        is_triggerable: 0,
    };
    let result: SVBError = unsafe { SVBGetCameraProperty(*id as c_int, &mut property) }.into();

    if result == SVBError::Success {
        Ok(property.into())
    } else {
        Err(result)
    }
}

pub fn get_camera_property_ex(id: i32) -> Result<SVBCameraPropertyEx, SVBError> {
    let mut property = LLSVBCameraPropertyEx {
        support_control_temp: 0,
        unused: [0; 64],
    };
    let result: SVBError = unsafe { SVBGetCameraPropertyEx(id as c_int, &mut property) }.into();

    if result == SVBError::Success {
        Ok(property.into())
    } else {
        Err(result)
    }
}

pub fn get_camera_info(id: usize) -> Result<SVBCameraInfo, SVBError> {
    let mut info = LLSVBCameraInfo {
        friendly_name: [0; 32],
        serial_number: [0; 32],
        port_type: [0; 32],
        device_id: 0,
        camera_id: 0,
    };
    let result: SVBError = unsafe { SVBGetCameraInfo(&mut info, id as c_int) }.into();

    if result == SVBError::Success {
        Ok(info.into())
    } else {
        Err(result)
    }
}

pub fn set_roi_format(
    id: i32,
    startx: i32,
    starty: i32,
    width: i32,
    height: i32,
    bin: i32,
) -> Result<(), SVBError> {
    let result: SVBError =
        unsafe { SVBSetROIFormat(id, startx, starty, width, height, bin) }.into();
    result.into()
}

pub fn set_roi_format_ex(
    id: i32,
    startx: i32,
    starty: i32,
    width: i32,
    height: i32,
    bin: i32,
    mode: i32,
) -> Result<(), SVBError> {
    let result: SVBError =
        unsafe { SVBSetROIFormatEx(id, startx, starty, width, height, bin, mode) }.into();
    result.into()
}

pub fn get_roi_format(id: i32) -> Result<(i32, i32, i32, i32, i32), SVBError> {
    let mut startx = 0;
    let mut starty = 0;
    let mut width = 0;
    let mut height = 0;
    let mut bin = 0;
    let result = unsafe {
        SVBGetROIFormat(
            id,
            &mut startx,
            &mut starty,
            &mut width,
            &mut height,
            &mut bin,
        )
    }
    .into();
    if result == SVBError::Success {
        Ok((startx, starty, width, height, bin))
    } else {
        Err(result)
    }
}

pub fn get_roi_format_ex(id: i32) -> Result<(i32, i32, i32, i32, i32, i32), SVBError> {
    let mut startx = 0;
    let mut starty = 0;
    let mut width = 0;
    let mut height = 0;
    let mut bin = 0;
    let mut mode = 0;
    let result = unsafe {
        SVBGetROIFormatEx(
            id,
            &mut startx,
            &mut starty,
            &mut width,
            &mut height,
            &mut bin,
            &mut mode,
        )
    }
    .into();
    if result == SVBError::Success {
        Ok((startx, starty, width, height, bin, mode))
    } else {
        Err(result)
    }
}

pub fn get_dropped_frames(id: &i32) -> Result<i32, SVBError> {
    let mut dropped_frames = 0;
    let result = unsafe { SVBGetDroppedFrames(*id, &mut dropped_frames) }.into();
    if result == SVBError::Success {
        Ok(dropped_frames)
    } else {
        Err(result)
    }
}

pub fn start_capture(id: &i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBStartVideoCapture(*id) }.into();
    result.into()
}

pub fn stop_capture(id: &i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBStopVideoCapture(*id) }.into();
    result.into()
}

pub trait PixelType {}
impl PixelType for u8 {}
impl PixelType for u16 {}
impl PixelType for [u8; 3] {}
impl PixelType for [u16; 3] {}
pub fn get_video_data<T>(
    id: &i32,
    buf: &mut [T],
    waitms: i32,
) -> Result<chrono::DateTime<chrono::Utc>, SVBError>
where
    T: PixelType + Sized,
{
    let size = std::mem::size_of_val(buf) as c_long;
    let result =
        unsafe { SVBGetVideoData(*id, buf.as_mut_ptr() as *mut c_uchar, size, waitms) }.into();
    if result == SVBError::Success {
        Ok(chrono::Utc::now())
    } else {
        Err(result)
    }
}

pub fn white_balance_once(id: i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBWhiteBalanceOnce(id) }.into();
    result.into()
}

pub fn send_soft_trigger(id: i32) -> Result<(), SVBError> {
    let result: SVBError = unsafe { SVBSendSoftTrigger(id) }.into();
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_number_of_connected_camera() {
        let num = get_number_of_connected_camera();
        println!("Number of connected camera: {}", num)
    }

    #[test]
    fn test_get_camera_info() {
        let num = get_number_of_connected_camera();
        if num > 0 {
            let info = get_camera_info(0).unwrap();
            println!("{:?}", info);
        }
    }

    #[test]
    fn test_openclose() {
        let num = get_number_of_connected_camera();
        if num > 0 {
            let info = get_camera_info(0).unwrap();
            println!("id = {}", info.camera_id);
            println!("opening camera");
            open_camera(info.camera_id).unwrap();
            println!("closing camera");
            close_camera(&info.camera_id).unwrap();
        }
    }

    #[test]
    fn test_queries() {
        let num = get_number_of_connected_camera();
        println!("{} connected cameras", num);
        if num > 0 {
            let info = get_camera_info(0).unwrap();
            println!("id = {}", info.camera_id);
            println!("opening camera");
            open_camera(info.camera_id).unwrap();
            let property = get_camera_property(&info.camera_id).unwrap();
            let property_ex = get_camera_property_ex(info.camera_id).unwrap();
            println!("{:?}", property);
            println!("{:?}", property_ex);

            // Try setting the gain
            set_control_value(&info.camera_id, SVBControlType::SVBExposure, 5000, false).unwrap();
            set_control_value(&info.camera_id, SVBControlType::SVBGain, 10, false).unwrap();
            let ncontrols = get_number_of_controls(&info.camera_id).unwrap();
            for idx in 0..ncontrols {
                let control = get_control_info(&info.camera_id, idx).unwrap();
                println!("ctrl = {:?}", control);
                let control_value =
                    get_control_value(&info.camera_id, control.control_type).unwrap();
                println!("value = {:?}", control_value);
            }
            println!("ncontrols = {}", ncontrols);

            println!(
                "pixel size = {} µm",
                get_pixel_size_microns(&info.camera_id).unwrap()
            );
            println!(
                "camera mode = {:?}",
                get_camera_mode(&info.camera_id).unwrap()
            );
            println!("ROI = {:?}", get_roi_format(info.camera_id).unwrap());
            println!("closing camera");
            close_camera(&info.camera_id).unwrap();
        }
    }

    #[test]
    fn test_frameout() {
        let num = get_number_of_connected_camera();
        if num > 0 {
            let info = get_camera_info(0).unwrap();
            println!("id = {}", info.camera_id);
            println!("opening camera");
            open_camera(info.camera_id).unwrap();
            let property = get_camera_property(&info.camera_id).unwrap();
            println!("{:?}", property);
            let mode = get_camera_mode(&info.camera_id).unwrap();
            println!("camera mode = {:?}", mode);
            let format = get_output_image_type(info.camera_id).unwrap();
            println!("output format = {:?}", format);
            let mut buf = vec![0u16; property.max_width as usize * property.max_height as usize];
            start_capture(&info.camera_id).unwrap();
            get_video_data(&info.camera_id, &mut buf, 1000).unwrap();
            stop_capture(&info.camera_id).unwrap();
            println!("closing camera");
            close_camera(&info.camera_id).unwrap();
            println!("Captured Frame");
            println!("frame = {:?}", &buf[0..100]);
            use std::io::Write;
            let mut f1 = std::fs::File::create("frame.dat").unwrap();
            f1.write_all(unsafe {
                std::slice::from_raw_parts(
                    buf.as_ptr() as *const u8,
                    buf.len() * std::mem::size_of::<u16>(),
                )
            })
            .expect("write must succeed");
        }
    }
}
