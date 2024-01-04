use anyhow::{anyhow, Result};
use log::{info, warn};
use std::{process::Command, str::from_utf8};

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
