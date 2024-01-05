use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Result};
use log::info;

use crate::utils;
use compositor_chromium::cef;

const ARM_MAC_TARGET: &str = "aarch64-apple-darwin";
const ARM_OUTPUT_FILE: &str = "video_compositor_darwin_aarch64.tar.gz";
const INTEL_MAC_TARGET: &str = "x86_64-apple-darwin";
const INTEL_OUTPUT_FILE: &str = "video_compositor_darwin_x86_64.tar.gz";

pub fn bundle_macos_app() -> Result<()> {
    if cfg!(target_arch = "x86_64") {
        bundle_app(INTEL_MAC_TARGET, INTEL_OUTPUT_FILE)?;
    } else if cfg!(target_arch = "aarch64") {
        bundle_app(ARM_MAC_TARGET, ARM_OUTPUT_FILE)?;
    } else {
        panic!("Unknown architecture")
    }
    Ok(())
}

fn bundle_app(target: &'static str, output_name: &'static str) -> Result<()> {
    tracing_subscriber::fmt().init();

    let root_dir_str = env!("CARGO_MANIFEST_DIR");
    let root_dir: PathBuf = root_dir_str.into();
    let build_dir = root_dir.join(format!("target/{target}/release"));
    let tmp_dir = root_dir.join("video_compositor");

    info!("Build main_process binary.");
    utils::cargo_build("main_process", target)?;
    info!("Build process_helper binary.");
    utils::cargo_build("process_helper", target)?;

    info!("Create macOS bundle.");
    cef::bundle_app(&build_dir, &tmp_dir.join("video_compositor.app"))?;

    fs::copy(
        build_dir.join("main_process"),
        tmp_dir.join("video_compositor"),
    )?;

    info!("Create tar.gz archive.");
    let exit_code = Command::new("tar")
        .args(["-C", root_dir_str, "-czvf", output_name, "video_compositor"])
        .spawn()?
        .wait()?
        .code();
    if exit_code != Some(0) {
        return Err(anyhow!("Command tar failed with exit code {:?}", exit_code));
    }

    Ok(())
}
