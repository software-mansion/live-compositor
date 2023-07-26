use std::fs;
use std::{env, error::Error, path::PathBuf};

use fs_extra::dir::{self, CopyOptions};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CEF_ROOT");
    
    let bindings = prepare()?;
    bindings.write_to_file(PathBuf::from(".").join("src").join("bindings.rs"))?;

    link()?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn prepare() -> Result<bindgen::Bindings, Box<dyn Error>> {
    let cef_root = env::var("CEF_ROOT")?;
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{cef_root}"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()?;

    let framework_out_path = PathBuf::from(env::var("OUT_DIR")?).join("Frameworks");
    let _ = fs::create_dir_all(&framework_out_path);

    let framework_path = PathBuf::from(cef_root)
        .join("Release")
        .join("Chromium Embedded Framework.framework");
    let options = CopyOptions {
        skip_exist: true,
        ..Default::default()
    };
    dir::copy(framework_path, framework_out_path, &options)?;
    Ok(bindings)
}

#[cfg(target_os = "macos")]
fn link() -> Result<(), Box<dyn Error>> {
    let out_path = PathBuf::from(env::var("OUT_DIR")?)
        .join("..")
        .join("..")
        .join("..");
    let dst = cmake::Config::new("CMakeLists.txt")
        .define("MAKE_BUILD_TYPE", "Debug")
        .define("OUT_DIR", out_path.display().to_string())
        .build();

    println!(r"cargo:rustc-link-search={}", dst.display());
    println!("cargo:rustc-link-lib=static=cef_dll_wrapper");

    Ok(())
}
