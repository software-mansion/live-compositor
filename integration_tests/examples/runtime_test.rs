use anyhow::{anyhow, Result};
use compositor_pipeline::{
    pipeline::{GraphicsContext, Options},
    Pipeline,
};
use compositor_render::{create_wgpu_ctx, error::ErrorStack, WgpuComponents};
use crossbeam_channel::Receiver;
use signal_hook::{consts, iterator::Signals};
use std::{
    io, process,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

use live_compositor::{
    config::{read_config, Config},
    logger::init_logger,
    server::run_api,
    state::ApiState,
};
use tracing::{error, info};

fn main() {
    println!("############### My pid is {}", process::id());
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    let config = read_config();

    // let WgpuComponents {
    //     instance,
    //     adapter,
    //     queue,
    //     device,
    // } = create_wgpu_ctx(
    //     config.force_gpu,
    //     config.required_wgpu_features,
    //     Default::default(),
    //     None,
    // )
    // .unwrap();
    //
    // let ctx = GraphicsContext {
    //     instance,
    //     adapter,
    //     queue,
    //     device,
    //     vulkan_ctx: None,
    // };

    let ctx = GraphicsContext::new(
        config.force_gpu,
        config.required_wgpu_features,
        Default::default(),
        None,
    )
    .unwrap();
    let runtime = Arc::new(Runtime::new().unwrap());

    init_logger(config.logger.clone());

    //let mut i = 0;
    //loop {
    //    i += 1;
    for i in 0..10000 {
        //for i in 0..1 {
        println!(">>>>>>>>>> ITERATION {i} <<<<<<<<<<<<");
        run_pipeline(config.clone(), runtime.clone(), ctx.clone())
    }

    // let _ = io::stdin().read_line(&mut input);
}

fn run_pipeline(config: Config, runtime: Arc<Runtime>, ctx: GraphicsContext) {
    // let runtime = Arc::new(Runtime::new().unwrap());
    let _ = Pipeline::new(Options {
        queue_options: config.queue_options,
        stream_fallback_timeout: config.stream_fallback_timeout,
        web_renderer: config.web_renderer,
        force_gpu: config.force_gpu,
        download_root: config.download_root,
        output_sample_rate: config.output_sample_rate,
        stun_servers: config.stun_servers,
        wgpu_features: config.required_wgpu_features,
        load_system_fonts: Some(true),
        wgpu_ctx: Some(ctx),
        tokio_rt: Some(runtime),
    })
    .unwrap_or_else(|err| {
        panic!(
            "Failed to start compositor.\n{}",
            ErrorStack::new(&err).into_string()
        )
    });
}

fn run_server() {
    println!("START");
    let (should_close_sender, should_close_receiver) = crossbeam_channel::bounded(1);
    thread::spawn(move || {
        run(should_close_receiver);
    });
    loop {
        match wait_for_server_ready(Duration::from_secs(20)) {
            Err(err) => {
                error!("{err}");
                process::exit(1);
            }
            Ok(_) => break,
        }
    }
    should_close_sender.send(()).unwrap();
    println!("DONE");
}

fn wait_for_server_ready(timeout: Duration) -> Result<()> {
    let server_status_url = "http://127.0.0.1:8081/status";
    let wait_start_time = Instant::now();
    loop {
        match reqwest::blocking::get(server_status_url) {
            Ok(_) => break,
            Err(_) => info!("Waiting for the server to start."),
        };
        if wait_start_time.elapsed() > timeout {
            return Err(anyhow!("Error while starting server, timeout exceeded."));
        }
        thread::sleep(Duration::from_millis(1000));
    }
    Ok(())
}

fn run(should_close_receiver: Receiver<()>) {
    listen_for_parent_termination();
    let config = read_config();

    info!("Starting LiveCompositor with config:\n{:#?}", config);
    let runtime = Arc::new(Runtime::new().unwrap());
    let (state, event_loop) = ApiState::new(config, runtime.clone()).unwrap_or_else(|err| {
        panic!(
            "Failed to start event loop.\n{}",
            ErrorStack::new(&err).into_string()
        )
    });

    thread::Builder::new()
        .name("HTTP server startup thread".to_string())
        .spawn(move || {
            if let Err(err) = run_api(state, runtime, should_close_receiver) {
                error!(%err);
                process::exit(1);
            }
        })
        .unwrap();
    let event_loop_fallback = || {
        let mut signals = Signals::new([consts::SIGINT]).unwrap();
        signals.forever().next();
    };
    if let Err(err) = event_loop.run_with_fallback(&event_loop_fallback) {
        panic!(
            "Failed to start event loop.\n{}",
            ErrorStack::new(&err).into_string()
        )
    }
}

#[cfg(target_os = "linux")]
fn listen_for_parent_termination() {
    use libc::{prctl, SIGTERM};
    unsafe {
        prctl(libc::PR_SET_PDEATHSIG, SIGTERM);
    }
}

#[cfg(target_os = "macos")]
fn listen_for_parent_termination() {
    use libc::SIGTERM;
    use std::{os::unix::process::parent_id, time::Duration};
    let ppid = parent_id();

    thread::Builder::new()
        .name("Parent process pid change".to_string())
        .spawn(move || loop {
            let current_pid = parent_id();
            if current_pid != ppid {
                info!("Compositor parent process was terminated.");
                unsafe {
                    libc::kill(std::process::id() as libc::c_int, SIGTERM);
                }
            }
            thread::sleep(Duration::from_secs(1));
        })
        .unwrap();
}
