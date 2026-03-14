# svbony

Safe Rust bindings for the [SVBony USB Camera SDK](https://www.svbony.com/).

This workspace provides two crates:

| Crate | Description |
|-------|-------------|
| **`svbony-sys`** | Low-level, unsafe FFI bindings to the SVBony C SDK (v1.13.4). Direct mapping of C types and functions. |
| **`svbony`** | Higher-level, safe Rust wrapper built on `svbony-sys`. Provides RAII camera handles (auto-close on drop), `Result`-based error handling, type-safe enums, and optional `image` crate integration. Most users should depend on this crate. |

## Supported Platforms

| OS | Architectures |
|----|---------------|
| macOS | aarch64 (Apple Silicon), x86_64 |
| Linux | x86_64, x86, armv7, aarch64 |
| Windows | x86_64, x86 |

## Prerequisites

1. Download the SVBony camera SDK from [svbony.com](https://www.svbony.com/downloads/software-driver)
2. Extract it and set the `SVBCAMERA_SDK_PATH` environment variable:

```bash
export SVBCAMERA_SDK_PATH=/path/to/SVBCameraSDK
```

The SDK directory must contain `include/SVBCameraSDK.h` and platform libraries under `lib/`.

## Building

```bash
cargo build --release
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
svbony = "0.1"
```

### Example

```rust
use svbony::{connected_cameras, Camera, ControlType};

fn main() -> svbony::Result<()> {
    // List connected cameras
    let cameras = connected_cameras()?;
    println!("Found {} camera(s)", cameras.len());

    if let Some(info) = cameras.first() {
        // Open camera (automatically closed on drop)
        let cam = Camera::open(info.camera_id)?;

        // Read camera properties
        let prop = cam.property()?;
        println!("Sensor: {}x{}", prop.max_width, prop.max_height);

        // Get/set controls
        let (gain, _auto) = cam.get_control(ControlType::Gain)?;
        println!("Current gain: {}", gain);

        // Capture a frame
        let roi = cam.roi()?;
        let mut buf = vec![0u8; (roi.width * roi.height) as usize];
        cam.start_capture()?;
        cam.get_frame(&mut buf, 1000)?;
        cam.stop_capture()?;
    }

    Ok(())
}
```

### Image feature

Enable the `image` feature to get frames as `image::DynamicImage`:

```toml
[dependencies]
svbony = { version = "0.1", features = ["image"] }
```

```rust
cam.start_capture()?;
let img = cam.get_image(1000)?;
img.save("frame.png")?;
cam.stop_capture()?;
```

## MSRV

Rust 1.68 or later.

## License

MIT