fn main() {
    println!("cargo::rerun-if-changed=renderer/src");

    let status = std::process::Command::new("bun")
        .args(["run", "build:sidecar"])
        .current_dir("renderer")
        .status()
        .expect("Failed to build sidecar");

    if !status.success() {
        panic!("Sidecar build failed");
    }

    let target_dir = std::env::var("OUT_DIR")
        .map(|d| {
            std::path::PathBuf::from(d)
                .ancestors()
                .nth(3)
                .unwrap()
                .to_path_buf()
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("target/debug"));

    std::fs::copy("renderer/dist/sidecar", target_dir.join("sidecar"))
        .expect("Failed to copy sidecar");

    #[cfg(feature = "soulver")]
    {
        println!("cargo::rerun-if-changed=SoulverWrapper/Sources");

        let soulver_status = std::process::Command::new("swift")
            .args(["build", "-c", "release"])
            .current_dir("SoulverWrapper")
            .status()
            .expect("Failed to build SoulverWrapper");

        if !soulver_status.success() {
            panic!("SoulverWrapper build failed");
        }

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let soulver_lib = format!(
            "{}/SoulverWrapper/.build/x86_64-unknown-linux-gnu/release",
            manifest_dir
        );
        let soulver_core = format!("{}/SoulverWrapper/Vendor/SoulverCore-linux", manifest_dir);

        println!("cargo:rustc-link-search=native={}", soulver_lib);
        println!("cargo:rustc-link-lib=dylib=SoulverWrapper");

        println!("cargo:rustc-link-search=native={}", soulver_core);
        println!("cargo:rustc-link-lib=dylib=SoulverCoreDynamic");

        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", soulver_lib);
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", soulver_core);
    }
}
