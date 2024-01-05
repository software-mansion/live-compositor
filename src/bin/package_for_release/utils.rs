use anyhow::{anyhow, Result};
use log::{info, warn};
use std::{fs, path::PathBuf, process::Command, str::from_utf8};

pub fn cargo_build(
    bin: &'static str,
    target: &'static str,
    disable_default_features: bool,
) -> Result<()> {
    let mut args = vec![
        "build",
        "--release",
        "--target",
        target,
        "--locked",
        "--bin",
        bin,
    ];
    if disable_default_features {
        args.extend(["--no-default-features"]);
    }

    info!("Running command \"cargo {}\"", args.join(" "));
    let output = Command::new("cargo")
        .args(args)
        .spawn()?
        .wait_with_output()?;
    if !output.status.success() {
        warn!("stdout: {:?}", &from_utf8(&output.stdout));
        warn!("stderr: {:?}", &from_utf8(&output.stderr));
        return Err(anyhow!("Command failed with exit code {}.", output.status));
    }
    Ok(())
}

pub fn setup_bundle_dir(dir: &PathBuf) -> Result<()> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(anyhow!("Expected directory path"));
        }

        info!("Bundle directory already exists. Removing...");
        fs::remove_dir_all(dir)?;
    }

    info!("Creating new bundle directory");
    fs::create_dir_all(dir)?;

    Ok(())
}
