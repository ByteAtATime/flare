fn main() {
    println!("cargo::rerun-if-changed=renderer/src");

    let status = std::process::Command::new("bun")
        .args(["run", "build"])
        .current_dir("renderer")
        .status()
        .expect("Failed to build renderer code");
}
