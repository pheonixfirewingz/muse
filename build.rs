use std::process::Command;
use std::env;

fn main() {
    let project_root = env::var("CARGO_MANIFEST_DIR").expect("Failed to get project root");

    let status = Command::new("./tools/tailwindcss")
        .args(["-i", "./assets/tailwind.css.template", "-o", "./assets/tailwind.css"])
        .current_dir(&project_root) // Run in project root
        .spawn()
        .expect("Failed to start Tailwind CSS build command")
        .wait()
        .expect("Failed to execute Tailwind CSS build command");

    if status.success() {
        println!("✅ Tailwind CSS built successfully.");
    } else {
        panic!("❌ Tailwind CSS build command failed");
    }

    println!("cargo:rerun-if-changed=assets/tailwind.css.template");
}
