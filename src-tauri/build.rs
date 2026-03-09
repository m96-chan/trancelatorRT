fn main() {
    // When cross-compiling for Android, whisper-rs-sys cannot run
    // the target preprocessor for bindgen. Use pre-generated bindings.
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        println!("cargo:rustc-env=WHISPER_DONT_GENERATE_BINDINGS=1");
    }

    tauri_build::build()
}
