# svbony

Safe, idiomatic Rust wrapper for the [SVBony](https://www.svbony.com/) USB
camera SDK. Provides RAII camera handles, Rust enums, and `Result`-based error
handling over the C SDK.

## SDK Installation

The SVBony Camera SDK is closed-source and **not bundled** with this crate.
Download it from [SVBony's download page](https://www.svbony.com/software-driver/)
and set an environment variable pointing to the SDK root:

```sh
export SVBCAMERA_SDK_PATH=/path/to/SVBCameraSDK
cargo build
```

See [`svbony-sys`](https://crates.io/crates/svbony-sys) for platform support
details.

## Quick Start

```rust,no_run
use svbony::*;

// List connected cameras
let cameras = connected_cameras().unwrap();
for info in &cameras {
    println!("{}: {}", info.camera_id, info.name);
}

// Open the first camera
let cam = Camera::open(cameras[0].camera_id).unwrap();
let prop = cam.property().unwrap();
println!("Sensor: {}x{}", prop.max_width, prop.max_height);

// Configure and capture
cam.set_roi(&RoiFormat {
    start_x: 0, start_y: 0,
    width: prop.max_width as i32,
    height: prop.max_height as i32,
    bin: 1,
}).unwrap();
cam.set_mode(CameraMode::Normal).unwrap();
cam.set_output_image_type(ImageType::Raw16).unwrap();
cam.set_control(ControlType::Exposure, 10_000, false).unwrap();

cam.start_capture().unwrap();

let bpp = ImageType::Raw16.bytes_per_pixel();
let mut buf = vec![0u8; prop.max_width as usize * prop.max_height as usize * bpp];
cam.get_frame(&mut buf, 5000).unwrap();

cam.stop_capture().unwrap();
// Camera is automatically closed when `cam` is dropped.
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `image` | off     | Adds `Camera::get_image()` returning an [`image::DynamicImage`](https://docs.rs/image). |

### Using the `image` feature

```toml
[dependencies]
svbony = { version = "0.1", features = ["image"] }
```

```rust,no_run
use svbony::*;

let cam = Camera::open(0).unwrap();
// ... configure ROI, mode, format ...
cam.start_capture().unwrap();
let img = cam.get_image(5000).unwrap();
cam.stop_capture().unwrap();

img.save("frame.png").unwrap();
```

Pixel format mapping:

| `ImageType`            | `DynamicImage` variant |
|------------------------|------------------------|
| `Raw8`, `Y8`           | `ImageLuma8`           |
| `Raw10`..`Raw16`, `Y10`..`Y16` | `ImageLuma16` |
| `Rgb24`                | `ImageRgb8`            |
| `Rgb32`                | `ImageRgba8`           |

## API Overview

### Free Functions

- `sdk_version()` -- SDK version string
- `connected_cameras()` -- list all connected cameras

### `Camera` (RAII)

**Open / close**: `Camera::open(id)`, auto-close on drop

**Properties**: `property()`, `property_ex()`, `firmware_version()`,
`serial_number()`, `pixel_size()`, `supported_modes()`, `needs_upgrade()`

**Controls**: `num_controls()`, `control_caps(index)`, `get_control(type)`,
`set_control(type, value, auto)`

**Image format**: `output_image_type()`, `set_output_image_type()`,
`roi()`, `set_roi()`, `roi_ex()`, `set_roi_ex()`

**Capture**: `start_capture()`, `stop_capture()`, `get_frame(buf, timeout)`,
`get_image(timeout)` *(feature = "image")*, `dropped_frames()`

**Mode / trigger**: `mode()`, `set_mode()`, `send_soft_trigger()`,
`set_trigger_output()`, `get_trigger_output()`

**Guide / misc**: `pulse_guide()`, `can_pulse_guide()`,
`white_balance_once()`, `set_auto_save()`, `restore_defaults()`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

**Note**: This crate does not bundle the SVBony SDK. The SDK itself is
proprietary; refer to SVBony's terms for its usage conditions.
