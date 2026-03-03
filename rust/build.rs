use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Validate the target is supported.
    match target_os.as_str() {
        "linux" | "macos" => {}
        "windows" if target_env == "gnu" => {}
        "windows" => {
            panic!(
                "ERROR: Only the `gnu` target environment is supported on Windows.\n\
                 Use `--target x86_64-pc-windows-gnu` (or install the corresponding rustup target)."
            );
        }
        other => {
            panic!(
                "ERROR: Unsupported target OS `{other}`. Supported: linux, macos, windows (gnu)."
            );
        }
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Cargo.toml is at the repo root, so CARGO_MANIFEST_DIR is the Go root.
    let go_root = &manifest_dir;

    // Check that `go` is on PATH.
    let go_version = Command::new("go")
        .arg("version")
        .output()
        .expect("ERROR: `go` not found on PATH. Install Go 1.24+ from https://go.dev/dl/");
    assert!(
        go_version.status.success(),
        "`go version` failed: {}",
        String::from_utf8_lossy(&go_version.stderr)
    );

    let lib_path = out_dir.join("libtsgo.a");

    let mut go_build = Command::new("go");
    go_build
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(&lib_path)
        .arg("./cmd/libtsgo")
        .current_dir(go_root)
        .env("CGO_ENABLED", "1");

    // // On windows-gnu, ensure cgo uses GCC (not MSVC or clang).
    // if target_os == "windows" {
    //     go_build.env("CC", "gcc");
    // }

    let status = go_build.status().expect("failed to run `go build`");
    assert!(status.success(), "`go build -buildmode=c-archive` failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=tsgo");

    // Platform-specific system libraries required by the Go runtime.
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=pthread");
            println!("cargo:rustc-link-lib=dylib=m");
            println!("cargo:rustc-link-lib=dylib=resolv");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=dylib=pthread");
            println!("cargo:rustc-link-lib=dylib=m");
            println!("cargo:rustc-link-lib=dylib=resolv");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=ntdll");
            println!("cargo:rustc-link-lib=dylib=ws2_32");
            println!("cargo:rustc-link-lib=dylib=winmm");
            println!("cargo:rustc-link-lib=dylib=userenv");
        }
        _ => panic!("ERROR: No system libraries configured for target OS `{target_os}`."),
    }

    println!("cargo:rerun-if-changed=cmd/libtsgo/main.go");
    println!("cargo:rerun-if-changed=go.mod");
    println!("cargo:rerun-if-changed=go.sum");
}
