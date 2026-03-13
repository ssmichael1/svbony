use std::env;
use std::path::PathBuf;

fn main() {
    let sdk_path = PathBuf::from(
        env::var("SVBCAMERA_SDK_PATH")
            .expect("SVBCAMERA_SDK_PATH environment variable must be set to the SDK root directory"),
    );

    if !sdk_path.exists() {
        panic!(
            "SVBCAMERA_SDK_PATH points to non-existent directory: {}",
            sdk_path.display()
        );
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let lib_subdir = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => "lib/arm64",
        ("macos", "x86_64") => "lib/x64",
        ("linux", "x86_64") => "lib/x64",
        ("linux", "x86") => "lib/x86",
        ("linux", "arm") => "lib/armv7",
        ("linux", "aarch64") => "lib/armv8",
        ("windows", "x86_64") => "lib/x64",
        ("windows", "x86") => "lib/x86",
        _ => panic!("Unsupported target: {target_os}-{target_arch}"),
    };

    let lib_dir = sdk_path.join(lib_subdir);
    if !lib_dir.exists() {
        panic!(
            "SDK library directory not found: {}",
            lib_dir.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=static=SVBCameraSDK");
            // The SDK ships libusb-1.0.0.dylib (the libusb-1.0.dylib symlink is
            // broken after zip extraction, so we link the versioned name directly).
            println!("cargo:rustc-link-lib=dylib=usb-1.0.0");
            // Frameworks required by the static SDK library
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=c++");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=static=SVBCameraSDK");
            println!("cargo:rustc-link-lib=dylib=usb-1.0");
            println!("cargo:rustc-link-lib=stdc++");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=SVBCameraSDK");
        }
        _ => unreachable!(),
    }

    // Re-run if the env var changes
    println!("cargo:rerun-if-env-changed=SVBCAMERA_SDK_PATH");
}
