# svbony-sys

Raw FFI bindings to the [SVBony](https://www.svbony.com/) USB camera C SDK (v1.13.4).

This crate provides unsafe, low-level access to all SDK functions, structs, and
constants. For a safe, idiomatic Rust API, use the [`svbony`](https://crates.io/crates/svbony)
crate instead.

## SDK Installation

The SVBony Camera SDK is closed-source and **not bundled** with this crate. You
must download it yourself from [SVBony's download page](https://www.svbony.com/downloads/software-driver)
and set an environment variable pointing to the SDK root:

```sh
export SVBCAMERA_SDK_PATH=/path/to/SVBCameraSDK
cargo build
```

The SDK root is the directory containing `include/` and `lib/`.

## Platform Support

| Target OS | Architecture | SDK lib subdirectory |
|-----------|-------------|---------------------|
| macOS     | aarch64     | `lib/arm64`         |
| macOS     | x86_64      | `lib/x64`           |
| Linux     | x86_64      | `lib/x64`           |
| Linux     | x86         | `lib/x86`           |
| Linux     | armv7       | `lib/armv7`         |
| Linux     | aarch64     | `lib/armv8`         |
| Windows   | x86_64      | `lib/x64`           |
| Windows   | x86         | `lib/x86`           |

### Linking

- **macOS / Linux**: Links `libSVBCameraSDK.a` statically, plus `libusb-1.0`
  dynamically. On macOS the IOKit and CoreFoundation frameworks are also linked.
- **Windows**: Links `SVBCameraSDK.dll` dynamically.

## Usage

Most users should depend on the safe [`svbony`](https://crates.io/crates/svbony)
crate. Use `svbony-sys` directly only if you need access to the raw C API:

```rust,no_run
use svbony_sys::*;
use std::os::raw::c_int;

let n: c_int = unsafe { SVBGetNumOfConnectedCameras() };
println!("{n} cameras connected");
```

## License

Licensed under the MIT license ([LICENSE](../LICENSE) or <http://opensource.org/licenses/MIT>).

**Note**: This crate does not bundle the SVBony SDK. The SDK itself is
proprietary; refer to SVBony's terms for its usage conditions.
