use anyhow::{anyhow, Result};
use fs_extra::dir::{self, CopyOptions};
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::utils;

const X86_TARGET: &str = "x86_64-unknown-linux-gnu";
const X86_OUTPUT_FILE: &str = "live_compositor_linux_x86_64.tar.gz";
const X86_WITH_WEB_RENDERER_OUTPUT_FILE: &str =
    "live_compositor_with_web_renderer_linux_x86_64.tar.gz";

const ARM_TARGET: &str = "aarch64-unknown-linux-gnu";
const ARM_OUTPUT_FILE: &str = "live_compositor_linux_aarch64.tar.gz";

pub fn bundle_linux_app() -> Result<()> {
    tracing_subscriber::fmt().init();

    if cfg!(target_arch = "x86_64") {
        bundle_app(X86_TARGET, X86_OUTPUT_FILE, false)?;
        bundle_app(X86_TARGET, X86_WITH_WEB_RENDERER_OUTPUT_FILE, true)?;
    } else if cfg!(target_arch = "aarch64") {
        bundle_app(ARM_TARGET, ARM_OUTPUT_FILE, false)?;
    }
    Ok(())
}

fn bundle_app(
    target_name: &'static str,
    output_name: &str,
    enable_web_rendering: bool,
) -> Result<()> {
    if enable_web_rendering {
        info!("Bundling compositor with web rendering");
    } else {
        info!("Bundling compositor without web rendering");
    }

    let root_dir_str = env!("CARGO_MANIFEST_DIR");
    let root_dir: PathBuf = root_dir_str.into();
    let release_dir = root_dir.join(format!("target/{target_name}/release"));
    let tmp_dir = root_dir.join("live_compositor");
    utils::setup_bundle_dir(&tmp_dir)?;

    info!("Build main_process binary.");
    utils::cargo_build("main_process", target_name, !enable_web_rendering)?;

    if enable_web_rendering {
        info!("Build process_helper binary.");
        utils::cargo_build("process_helper", target_name, false)?;

        info!("Create {} directory", tmp_dir.display());
        fs::create_dir_all(tmp_dir.clone())?;

        info!("Copy main_process binary.");
        fs::copy(
            release_dir.join("main_process"),
            tmp_dir.join("live_compositor_main"),
        )?;

        info!("Copy process_helper binary.");
        fs::copy(
            release_dir.join("process_helper"),
            tmp_dir.join("live_compositor_process_helper"),
        )?;

        info!("Copy wrapper script.");
        fs::copy(
            root_dir.join("src/bin/package_for_release/linux_runtime_wrapper.sh"),
            tmp_dir.join("live_compositor"),
        )?;

        info!(
            "Copy lib directory. {:?} {:?}",
            release_dir.join("lib"),
            tmp_dir.join("lib"),
        );

        dir::copy(release_dir.join("lib"), tmp_dir, &CopyOptions::default())?;
    } else {
        info!("Copy main_process binary.");
        fs::copy(
            release_dir.join("main_process"),
            tmp_dir.join("live_compositor"),
        )?;
    }

    info!("Create tar.gz archive.");
    let exit_code = Command::new("tar")
        .args(["-C", root_dir_str, "-czvf", output_name, "live_compositor"])
        .spawn()?
        .wait()?
        .code();
    if exit_code != Some(0) {
        return Err(anyhow!("Command tar failed with exit code {:?}", exit_code));
    }

    Ok(())
}
