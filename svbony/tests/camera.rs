//! Integration tests requiring a physical SVBony camera.
//!
//! All tests are `#[ignore]`d by default so `cargo test` passes without
//! hardware. Run them with:
//!
//! ```sh
//! cargo test --features image -p svbony -- --ignored --nocapture
//! ```
//!
//! A single test at a time is safest since the camera is a shared resource:
//!
//! ```sh
//! cargo test --features image -p svbony -- --ignored --nocapture --test-threads=1
//! ```

use svbony::*;

/// Helper: open the first connected camera or skip.
fn open_first_camera() -> Camera {
    let cameras = connected_cameras().expect("failed to enumerate cameras");
    assert!(!cameras.is_empty(), "no SVBony camera connected");
    let info = &cameras[0];
    eprintln!("Using camera: {} (id={})", info.name, info.camera_id);
    Camera::open(info.camera_id).expect("failed to open camera")
}

#[test]
#[ignore]
fn enumerate_cameras() {
    let cameras = connected_cameras().expect("failed to enumerate cameras");
    assert!(!cameras.is_empty(), "no SVBony camera connected");
    for cam in &cameras {
        eprintln!(
            "  id={} name={:?} serial={:?} port={:?}",
            cam.camera_id, cam.name, cam.serial, cam.port_type
        );
    }
}

#[test]
#[ignore]
fn sdk_version_nonempty() {
    let ver = sdk_version();
    eprintln!("SDK version: {ver}");
    assert!(!ver.is_empty());
}

#[test]
#[ignore]
fn camera_properties() {
    let cam = open_first_camera();

    let prop = cam.property().expect("property");
    eprintln!("Sensor: {}x{}", prop.max_width, prop.max_height);
    eprintln!("Color: {}, Bayer: {:?}", prop.is_color, prop.bayer_pattern);
    eprintln!("Bins: {:?}", prop.supported_bins);
    eprintln!("Formats: {:?}", prop.supported_formats);
    eprintln!("Max bit depth: {}", prop.max_bit_depth);
    eprintln!("Trigger cam: {}", prop.is_trigger_cam);
    assert!(prop.max_width > 0);
    assert!(prop.max_height > 0);
    assert!(!prop.supported_bins.is_empty());
    assert!(!prop.supported_formats.is_empty());

    let prop_ex = cam.property_ex().expect("property_ex");
    eprintln!(
        "Pulse guide: {}, Temp control: {}",
        prop_ex.supports_pulse_guide, prop_ex.supports_temp_control
    );

    let fw = cam.firmware_version().expect("firmware_version");
    eprintln!("Firmware: {fw}");

    let sn = cam.serial_number().expect("serial_number");
    eprintln!("Serial (raw): {:?}", &sn[..16]);

    if let Ok(px) = cam.pixel_size() {
        eprintln!("Pixel size: {px} um");
    }
}

#[test]
#[ignore]
fn list_controls() {
    let cam = open_first_camera();
    let n = cam.num_controls().expect("num_controls");
    eprintln!("{n} controls:");
    for i in 0..n {
        let caps = cam.control_caps(i).expect("control_caps");
        let (val, auto) = cam
            .get_control(caps.control_type)
            .expect("get_control");
        eprintln!(
            "  {:?} ({}) = {val} (auto={auto}) [{} .. {}] default={}{}",
            caps.control_type,
            caps.name,
            caps.min_value,
            caps.max_value,
            caps.default_value,
            if caps.is_writable { "" } else { " [RO]" },
        );
    }
}

#[test]
#[ignore]
fn set_and_read_gain() {
    let cam = open_first_camera();

    // Full SDK init sequence: open → ROI → mode → capture.
    // SetControlValue can return GeneralError if the camera hasn't been
    // fully initialized (especially after a previous unclean shutdown).
    let prop = cam.property().expect("property");
    cam.set_roi(&RoiFormat {
        start_x: 0,
        start_y: 0,
        width: prop.max_width as i32,
        height: prop.max_height as i32,
        bin: 1,
    })
    .expect("set_roi");
    cam.set_mode(CameraMode::Normal).expect("set_mode");
    cam.start_capture().expect("start_capture");

    let (current, _) = cam.get_control(ControlType::Gain).expect("get_control");
    eprintln!("Gain before: {current}");

    // Set to a different value so we can verify the write
    let target = if current != 10 { 10 } else { 20 };
    eprintln!("Setting gain to {target}");
    cam.set_control(ControlType::Gain, target, false)
        .expect("set_control");

    let (val, auto) = cam.get_control(ControlType::Gain).expect("get_control");
    eprintln!("Gain after set: {val} (auto={auto})");
    assert_eq!(val, target);
    assert!(!auto);

    cam.stop_capture().expect("stop_capture");
}

#[test]
#[ignore]
fn capture_raw_frame() {
    let cam = open_first_camera();

    // Use default ROI (full sensor) and Raw8
    cam.set_output_image_type(ImageType::Raw8)
        .expect("set_output_image_type");
    let roi = cam.roi().expect("roi");
    eprintln!("ROI: {}x{} @ bin={}", roi.width, roi.height, roi.bin);

    let buf_size = roi.width as usize * roi.height as usize;
    let mut buf = vec![0u8; buf_size];

    cam.start_capture().expect("start_capture");
    cam.get_frame(&mut buf, 5000).expect("get_frame");
    cam.stop_capture().expect("stop_capture");

    // Sanity: not all zeros
    let nonzero = buf.iter().filter(|&&b| b != 0).count();
    eprintln!(
        "Frame: {} bytes, {nonzero} non-zero ({:.1}%)",
        buf.len(),
        nonzero as f64 / buf.len() as f64 * 100.0
    );
    assert!(nonzero > 0, "frame is all zeros");

    let dropped = cam.dropped_frames().expect("dropped_frames");
    eprintln!("Dropped frames: {dropped}");
}

#[cfg(feature = "image")]
#[test]
#[ignore]
fn capture_to_png() {
    use std::path::PathBuf;

    let cam = open_first_camera();
    let prop = cam.property().expect("property");

    // Use Y8 for a clean grayscale PNG
    cam.set_output_image_type(ImageType::Y8)
        .expect("set_output_image_type");

    // Short exposure for a quick test frame
    cam.set_control(ControlType::Exposure, 50_000, false)
        .expect("set exposure");

    cam.start_capture().expect("start_capture");
    let img = cam.get_image(5000).expect("get_image");
    cam.stop_capture().expect("stop_capture");

    eprintln!("Image: {}x{}", img.width(), img.height());
    assert_eq!(img.width(), prop.max_width as u32);
    assert_eq!(img.height(), prop.max_height as u32);

    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("test_frame.png");
    img.save(&path).expect("save PNG");
    eprintln!("Saved to {}", path.display());

    let metadata = std::fs::metadata(&path).expect("file metadata");
    assert!(metadata.len() > 0, "PNG file is empty");
    eprintln!("PNG size: {} bytes", metadata.len());
}

#[cfg(feature = "image")]
#[test]
#[ignore]
fn capture_16bit_to_png() {
    use std::path::PathBuf;

    let cam = open_first_camera();
    let prop = cam.property().expect("property");

    // Pick the highest bit-depth mono format available
    let format = if prop.supported_formats.contains(&ImageType::Raw16) {
        ImageType::Raw16
    } else if prop.supported_formats.contains(&ImageType::Y16) {
        ImageType::Y16
    } else {
        eprintln!("No 16-bit format available, skipping");
        return;
    };
    eprintln!("Using format: {format:?}");

    cam.set_output_image_type(format)
        .expect("set_output_image_type");
    cam.set_control(ControlType::Exposure, 100_000, false)
        .expect("set exposure");

    cam.start_capture().expect("start_capture");
    let img = cam.get_image(5000).expect("get_image");
    cam.stop_capture().expect("stop_capture");

    assert!(img.as_luma16().is_some(), "expected Luma16 image");
    eprintln!("Image: {}x{} Luma16", img.width(), img.height());

    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("test_frame_16bit.png");
    img.save(&path).expect("save PNG");
    eprintln!("Saved to {}", path.display());

    let metadata = std::fs::metadata(&path).expect("file metadata");
    assert!(metadata.len() > 0, "PNG file is empty");
    eprintln!("PNG size: {} bytes", metadata.len());
}

#[test]
#[ignore]
fn camera_modes() {
    let cam = open_first_camera();
    let prop = cam.property().expect("property");
    if !prop.is_trigger_cam {
        eprintln!("Not a trigger camera, skipping mode tests");
        return;
    }

    let modes = cam.supported_modes().expect("supported_modes");
    eprintln!("Supported modes: {modes:?}");
    assert!(!modes.is_empty());

    let current = cam.mode().expect("mode");
    eprintln!("Current mode: {current:?}");

    // Set back to normal to leave camera in a clean state
    cam.set_mode(CameraMode::Normal).expect("set_mode Normal");
}

#[test]
#[ignore]
fn pulse_guide_support() {
    let cam = open_first_camera();
    let can = cam.can_pulse_guide().expect("can_pulse_guide");
    eprintln!("Pulse guide supported: {can}");

    let prop_ex = cam.property_ex().expect("property_ex");
    assert_eq!(can, prop_ex.supports_pulse_guide);
}

#[test]
#[ignore]
fn firmware_upgrade_check() {
    let cam = open_first_camera();
    let (needs, version) = cam.needs_upgrade().expect("needs_upgrade");
    eprintln!("Needs upgrade: {needs}, min version: {version:?}");
}
