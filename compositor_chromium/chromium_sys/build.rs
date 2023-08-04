use anyhow::{Context, Result};
use bindgen::callbacks::ParseCallbacks;
use fs_extra::dir::{self, CopyOptions};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CEF_ROOT");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let cef_root = if let Ok(cef_root) = env::var("CEF_ROOT") {
        PathBuf::from(cef_root)
    } else {
        out_dir.join("cef_root")
    };

    if !cef_root.exists() {
        download_cef(&cef_root);
    }

    let target_path = out_dir
        .parent()
        .context("chromium_sys build directory not found")?
        .parent()
        .context("deps build directory not found")?
        .parent()
        .context("target build directory not found")?;
    let bindings = prepare(&cef_root, &target_path)?;
    bindings.write_to_file(PathBuf::from(".").join("src").join("bindings.rs"))?;

    link(&cef_root, &target_path);

    Ok(())
}

#[allow(unused_variables)]
fn prepare(cef_root: &Path, target_path: &Path) -> Result<bindgen::Bindings> {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", cef_root.display().to_string()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(RemoveCommentsCallback))
        .generate()?;

    #[cfg(target_os = "macos")]
    {
        let framework_out_path = target_path.join("Frameworks");
        let _ = std::fs::create_dir_all(&framework_out_path);
        let framework_path = PathBuf::from(cef_root)
            .join("Release")
            .join("Chromium Embedded Framework.framework");
        let options = CopyOptions {
            skip_exist: true,
            ..Default::default()
        };

        dir::copy(framework_path, framework_out_path, &options)?;
        dir::copy("resources", target_path, &options)?;
    }

    #[cfg(target_os = "linux")]
    {
        let options = CopyOptions {
            skip_exist: true,
            content_only: true,
            ..Default::default()
        };

        let cef_root = PathBuf::from(cef_root);
        let release_path = cef_root.join("Release");
        let resources_path = cef_root.join("Resources");
        let lib_path = target_path.join("lib");
        let _ = fs::create_dir_all(&lib_path);

        dir::copy(release_path, &lib_path, &options)?;
        dir::copy(resources_path, &lib_path, &options)?;
    }

    Ok(bindings)
}

#[cfg(target_os = "macos")]
fn link(cef_root: &Path, _target_path: &Path) {
    let build_type = if cfg!(debug_assertions) {
        "Debug"
    } else {
        "Release"
    };

    let dst = cmake::Config::new("CMakeLists.txt")
        .define("MAKE_BUILD_TYPE", build_type)
        .define("CEF_ROOT", cef_root.display().to_string())
        .build();

    println!("cargo:rustc-link-search={}", dst.display());
    println!("cargo:rustc-link-lib=static=cef_dll_wrapper");
}

#[cfg(target_os = "linux")]
fn link(cef_root: &Path, target_path: &Path) {
    println!(
        "cargo:rustc-link-search=native={}",
        PathBuf::from(cef_root).join("Release").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        target_path.join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=cef");
}

#[derive(Debug)]
struct RemoveCommentsCallback;

impl ParseCallbacks for RemoveCommentsCallback {
    fn process_comment(&self, _comment: &str) -> Option<String> {
        Some(String::new())
    }
}

fn download_cef(cef_root_path: &Path) {
    let platform = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "arm") {
            "macosarm64"
        } else {
            "macosx64"
        }
    } else if cfg!(target_os = "linux") {
        "linux64"
    } else {
        panic!("Unsupported platform");
    };

    let download_path = cef_root_path.parent().unwrap();
    let url = format!("https://cef-builds.spotifycdn.com/cef_binary_115.3.11%2Bga61da9b%2Bchromium-115.0.5790.114_{platform}_minimal.tar.bz2");
    let resp = reqwest::blocking::get(url).unwrap();

    let archive_name = "cef.tar.bz2";
    let content = resp.bytes().unwrap();
    fs::write(&download_path.join(archive_name), content).unwrap();

    let _ = fs::create_dir_all(&cef_root_path);

    let tar_status = Command::new("tar")
        .args([
            "-xvf",
            &download_path.join(archive_name).display().to_string(),
            "-C",
            &cef_root_path.display().to_string(),
            "--strip-components=1",
        ])
        .status()
        .unwrap();
    if !tar_status.success() {
        panic!("failed to unarchive CEF binaries");
    }
}
