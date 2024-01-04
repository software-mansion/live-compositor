use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Result};
use log::info;

use crate::utils;

const ARM_MAC_TARGET: &str = "aarch64-apple-darwin";
const INTEL_MAC_TARGET: &str = "x86_64-apple-darwin";

pub fn bundle_macos_app(enable_web_rendering: bool) -> Result<()> {
    let output_name = match enable_web_rendering {
        true => root_dir.join("video_compositor_with_web_rendering"),
        false => root_dir.join("video_compositor"),
    };

    if cfg!(target_arch = "x86_64") {
        bundle_app(
            INTEL_MAC_TARGET,
            format!("{output_name}_x86_64.tar.gz"),
            enable_web_rendering,
        )?;
    } else if cfg!(target_arch = "aarch64") {
        bundle_app(
            ARM_MAC_TARGET,
            format!("{output_name}_aarch64.tar.gz"),
            enable_web_rendering,
        )?;
    } else {
        panic!("Unknown architecture")
    }
    Ok(())
}

fn bundle_app(
    target: &'static str,
    output_name: &'static str,
    enable_web_rendering: bool,
) -> Result<()> {
    let root_dir_str = env!("CARGO_MANIFEST_DIR");
    let root_dir: PathBuf = root_dir_str.into();
    let build_dir = root_dir.join(format!("target/{target}/release"));
    let tmp_dir = root_dir.join("video_compositor");

    info!("Build main_process binary.");
    utils::cargo_build("main_process", target, !enable_web_rendering)?;

    info!("Create macOS bundle.");
    if enable_web_rendering {
        use compositor_chromium::cef;

        info!("Build process_helper binary.");
        utils::cargo_build("process_helper", target)?;
        cef::bundle_app(&build_dir, &tmp_dir.join("video_compositor.app"), false)?;
    } else {
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir)?;
    }

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
