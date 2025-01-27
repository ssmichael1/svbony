use super::types::*;

use std::ffi::{c_char, c_int, c_long, c_uint};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct LLSVBCameraInfo {
    pub friendly_name: [c_char; 32],
    pub serial_number: [c_char; 32],
    pub port_type: [c_char; 32],
    pub device_id: c_uint,
    pub camera_id: c_int,
}

#[derive(Debug, Clone)]
pub struct SVBCameraInfo {
    pub friendly_name: String,
    pub serial_number: String,
    pub port_type: String,
    pub device_id: u32,
    pub camera_id: i32,
}

impl From<LLSVBCameraInfo> for SVBCameraInfo {
    fn from(info: LLSVBCameraInfo) -> Self {
        let friendly_name =
            String::from_utf8(info.friendly_name.iter().map(|&c| c as u8).collect())
                .unwrap()
                .trim()
                .trim_matches(char::from(0))
                .to_string();
        let serial_number =
            String::from_utf8(info.serial_number.iter().map(|&c| c as u8).collect())
                .unwrap()
                .trim()
                .trim_matches(char::from(0))
                .to_string();
        let port_type = String::from_utf8(info.port_type.iter().map(|&c| c as u8).collect())
            .unwrap()
            .trim()
            .trim_matches(char::from(0))
            .to_string();
        SVBCameraInfo {
            friendly_name,
            serial_number,
            port_type,
            device_id: info.device_id,
            camera_id: info.camera_id,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct LLSVBCameraProperty {
    pub max_height: c_long,
    pub max_width: c_long,

    pub is_color_cam: c_int,
    pub bayer_pattern: c_int,
    pub supported_bins: [c_int; 16],
    pub supported_video_format: [c_int; 8],
    pub max_bit_depth: c_int,
    pub is_triggerable: c_int,
}

#[derive(Debug, Clone)]
pub struct SVBCameraProperty {
    pub max_height: i32,
    pub max_width: i32,
    pub is_color_cam: bool,
    pub bayer_pattern: SVBBayerPattern,
    pub supported_bins: Vec<i32>,
    pub supported_video_format: Vec<SVBImageType>,
    pub max_bit_depth: i32,
    pub is_triggerable: bool,
}

impl From<LLSVBCameraProperty> for SVBCameraProperty {
    fn from(property: LLSVBCameraProperty) -> Self {
        let supported_bins = property
            .supported_bins
            .iter()
            .take_while(|&&x| x != 0)
            .copied()
            .collect();
        let supported_video_format = property
            .supported_video_format
            .iter()
            .take_while(|&&x| x != -1)
            .map(|&x| SVBImageType::from(x))
            .collect();
        SVBCameraProperty {
            max_height: property.max_height as i32,
            max_width: property.max_width as i32,
            is_color_cam: property.is_color_cam != 0,
            bayer_pattern: property.bayer_pattern.into(),
            supported_bins,
            supported_video_format,
            max_bit_depth: property.max_bit_depth,
            is_triggerable: property.is_triggerable != 0,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct LLSVBCameraPropertyEx {
    pub support_control_temp: c_int,
    pub unused: [c_int; 64],
}

#[derive(Clone, Debug)]
pub struct SVBCameraPropertyEx {
    pub support_control_temp: bool,
}

impl From<LLSVBCameraPropertyEx> for SVBCameraPropertyEx {
    fn from(property: LLSVBCameraPropertyEx) -> Self {
        SVBCameraPropertyEx {
            support_control_temp: property.support_control_temp != 0,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct LLSVBControlCaps {
    pub name: [c_char; 64],
    pub description: [c_char; 128],
    pub max_value: c_long,
    pub min_value: c_long,
    pub default_value: c_long,
    pub is_auto_supported: c_int,
    pub is_writeable: c_int,
    pub control_type: c_int,
    pub unused: [c_char; 32],
}

#[derive(Debug, Clone)]
pub struct SVBControlCaps {
    pub name: String,
    pub description: String,
    pub max_value: i32,
    pub min_value: i32,
    pub default_value: i32,
    pub is_auto_supported: bool,
    pub is_writeable: bool,
    pub control_type: SVBControlType,
}

impl From<LLSVBControlCaps> for SVBControlCaps {
    fn from(caps: LLSVBControlCaps) -> Self {
        let name = String::from_utf8(caps.name.iter().map(|&c| c as u8).collect())
            .unwrap()
            .trim()
            .trim_matches(char::from(0))
            .to_string();
        let description = String::from_utf8(caps.description.iter().map(|&c| c as u8).collect())
            .unwrap()
            .trim()
            .trim_matches(char::from(0))
            .to_string();
        SVBControlCaps {
            name,
            description,
            max_value: caps.max_value as i32,
            min_value: caps.min_value as i32,
            default_value: caps.default_value as i32,
            is_auto_supported: caps.is_auto_supported != 0,
            is_writeable: caps.is_writeable != 0,
            control_type: SVBControlType::from(caps.control_type),
        }
    }
}

impl std::fmt::Display for SVBControlCaps {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "  Description: {}", self.description)?;
        writeln!(f, "  Max Value: {}", self.max_value)?;
        writeln!(f, "  Min Value: {}", self.min_value)?;
        writeln!(f, "  Default Value: {}", self.default_value)?;
        writeln!(f, "  Is Auto Supported: {}", self.is_auto_supported)?;
        writeln!(f, "  Is Writeable: {}", self.is_writeable)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SVBId {
    pub id: [c_char; 64],
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SVBSupportedMode {
    pub mode: [SVBCameraMode; 16],
}
