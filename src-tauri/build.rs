use std::process::Command;

fn main() {
    let _ = Command::new("patchelf")
        .arg("--set-rpath")
        .arg("$ORIGIN")
        .arg("SoulverWrapper/Vendor/SoulverCore-linux/libSoulverCoreDynamic.so")
        .status()
        .expect("Failed to patch elf for libSoulverCoreDynamic");

    println!("cargo:rustc-link-search=native=SoulverWrapper/.build/release");

    println!("cargo:rustc-link-lib=SoulverWrapper");

    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/raycast-linux/SoulverWrapper/.build/release");

    tauri_build::build();
}
