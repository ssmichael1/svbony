use std::env;
use std::path::Path;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    //println!("cargo:rustc-flags=-l dylib=c++");
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("SVBCameraSDK/lib/arm64").display()
    );
    println!("cargo:rustc-link-lib=static=SVBCameraSDK");
}
