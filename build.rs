fn main() {
    println!("cargo::rerun-if-changed=renderer/src");

    let status = std::process::Command::new("bun")
        .args(["run", "dev"])
        .current_dir("renderer")
        .status()
        .expect("Failed to build renderer code");
}
