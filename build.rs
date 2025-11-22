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
}
