use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Prefer pkg-config when available.
    let pc = Command::new("pkg-config")
        .args(["--cflags", "--libs", "realsense2"])
        .output();

    if let Ok(output) = pc {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("cargo:warning=realsense2: using pkg-config");
            for flag in stdout.split_whitespace() {
                if let Some(dir) = flag.strip_prefix("-I") {
                    println!("cargo:rustc-link-search=native={}", dir);
                } else if let Some(dir) = flag.strip_prefix("-L") {
                    println!("cargo:rustc-link-search=native={}", dir);
                }
            }
            println!("cargo:rustc-link-lib=dylib=realsense2");
        }
    }

    // Fallback: search common install locations.
    let candidates: &[&str] = if target_os == "macos" {
        &["/usr/local/lib", "/opt/homebrew/lib"]
    } else {
        &["/usr/local/lib", "/usr/lib/x86_64-linux-gnu", "/usr/lib/aarch64-linux-gnu"]
    };

    let mut found = false;
    for dir in candidates {
        let path = PathBuf::from(dir);
        if path.join("librealsense2.so").exists()
            || path.join("librealsense2.dylib").exists()
            || path.join("librealsense2.a").exists()
        {
            println!("cargo:rustc-link-search=native={}", dir);
            println!("cargo:rustc-link-lib=dylib=realsense2");
            found = true;
            break;
        }
    }

    if !found {
        println!("cargo:warning=realsense2: no librealsense2 library found via pkg-config or common paths");
    }
}
