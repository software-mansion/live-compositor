use log::info;

#[cfg(target_os = "linux")]
mod bundle_linux;
#[cfg(target_os = "macos")]
mod bundle_macos;
mod utils;

#[cfg(target_os = "linux")]
fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    info!("Bundling compositor with web rendering");
    bundle_linux::bundle_linux_app(true).unwrap();

    info!("Bundling compositor without web rendering");
    bundle_linux::bundle_linux_app(false).unwrap();
}

#[cfg(target_os = "macos")]
fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    info!("Bundling compositor with web rendering");
    bundle_macos::bundle_macos_app(true).unwrap();

    info!("Bundling compositor without web rendering");
    bundle_macos::bundle_macos_app(false).unwrap();
}
