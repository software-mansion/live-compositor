use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use clap::Parser;
use compositor_pipeline::{
    pipeline::{
        encoder::VideoEncoderOptions,
        input::{
            mp4::{Mp4Options, Source},
            InputOptions,
        },
        output::EncodedDataOutputOptions,
        GraphicsContext, Options, OutputVideoOptions, PipelineOutputEndCondition,
        RegisterInputOptions, RegisterOutputOptions,
    },
    queue::{self, QueueInputOptions, QueueOptions},
    Pipeline,
};

use compositor_pipeline::pipeline::encoder::ffmpeg_h264::Options as H264OutputOptions;
use compositor_render::{
    scene::{
        Component, HorizontalAlign, InputStreamComponent, RGBAColor, TilesComponent, VerticalAlign,
    },
    web_renderer::WebRendererInitOptions,
    Framerate, InputId, OutputId, Resolution,
};
use live_compositor::{
    config::{read_config, LoggerConfig},
    logger,
};
use tracing::warn;

mod args;

use args::{Args, Argument, SingleBenchConfig};

fn main() {
    let args = Args::parse();
    let config = read_config();
    ffmpeg_next::format::network::init();
    let logger_config = LoggerConfig {
        level: "compositor_pipeline=error,vk-video=info,benchmark=info".into(),
        ..config.logger
    };
    logger::init_logger(logger_config);

    let ctx = GraphicsContext::new(
        false,
        wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,
        Default::default(),
        None,
    )
    .unwrap();

    if cfg!(debug_assertions) {
        warn!("This benchmark is running in debug mode. Make sure to run in release mode for reliable results.");
    }

    let reports = run_args(ctx, &args);
    SingleBenchConfig::log_report_header();
    for report in reports {
        report.log_as_report();
    }
}

fn run_args(ctx: GraphicsContext, args: &Args) -> Vec<SingleBenchConfig> {
    let arguments = args.arguments();
    let mut reports = Vec::new();

    // check maximize count
    let maximize_count = arguments
        .iter()
        .filter(|arg| matches!(arg, Argument::Maximize))
        .count();

    if maximize_count > 1 {
        panic!("Only one argument can be set to 'maximize'");
    }

    run_args_iterate(ctx, args, arguments, &mut reports);

    reports
}

fn run_args_iterate(
    ctx: GraphicsContext,
    args: &Args,
    arguments: Box<[Argument]>,
    reports: &mut Vec<SingleBenchConfig>,
) -> bool {
    for (i, argument) in arguments.iter().enumerate() {
        if matches!(argument, Argument::IterateExp) {
            let mut any_succeeded = false;
            let mut count = 1;
            loop {
                let mut arguments = arguments.clone();
                arguments[i] = Argument::Constant(count);

                if run_args_iterate(ctx.clone(), args, arguments, reports) {
                    any_succeeded = true;
                    count *= 2;
                    continue;
                } else {
                    return any_succeeded;
                }
            }
        }
    }

    run_args_maximize(ctx, args, arguments, reports)
}

fn run_args_maximize(
    ctx: GraphicsContext,
    args: &Args,
    arguments: Box<[Argument]>,
    reports: &mut Vec<SingleBenchConfig>,
) -> bool {
    for (i, argument) in arguments.iter().enumerate() {
        if matches!(argument, Argument::Maximize) {
            let upper_bound = find_upper_bound(1, |count| {
                let mut arguments = arguments.clone();
                arguments[i] = Argument::Constant(count);
                let config = args.with_arguments(&arguments);
                config.log_running_config();
                run_single_test(ctx.clone(), config)
            });

            if upper_bound == 0 {
                return false;
            }

            let result = binsearch(upper_bound / 2, upper_bound, |count| {
                let mut arguments = arguments.clone();
                arguments[i] = Argument::Constant(count);
                let config = args.with_arguments(&arguments);
                config.log_running_config();
                run_single_test(ctx.clone(), config)
            });

            let mut arguments = arguments.clone();
            arguments[i] = Argument::Constant(result);
            reports.push(args.with_arguments(&arguments));
            return true;
        }
    }

    // if we got here, there is no maximize, so just run a single test
    let config = args.with_arguments(&arguments);
    run_single_test(ctx, config)
}

fn binsearch(mut start: u64, mut end: u64, test_fn: impl Fn(u64) -> bool) -> u64 {
    while start < end {
        let midpoint = (start + end + 1) / 2;

        if test_fn(midpoint) {
            start = midpoint;
        } else {
            end = midpoint - 1;
        }
    }

    end
}

fn find_upper_bound(start: u64, test_fn: impl Fn(u64) -> bool) -> u64 {
    let mut end = start;

    while test_fn(end) {
        end *= 2;
    }

    end - 1
}

/// true - works
/// false - too slow
fn run_single_test(ctx: GraphicsContext, bench_config: SingleBenchConfig) -> bool {
    let (pipeline, _event_loop) = Pipeline::new(Options {
        queue_options: QueueOptions {
            never_drop_output_frames: true,
            output_framerate: Framerate {
                num: bench_config.framerate as u32,
                den: 1,
            },
            default_buffer_duration: queue::DEFAULT_BUFFER_DURATION,
            ahead_of_time_processing: false,
            run_late_scheduled_events: true,
        },
        web_renderer: WebRendererInitOptions {
            enable: false,
            enable_gpu: false,
        },
        wgpu_ctx: Some(ctx),
        force_gpu: false,
        download_root: std::env::temp_dir(),
        wgpu_features: wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
            | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        load_system_fonts: Some(false),
        output_sample_rate: 48_000,
        stream_fallback_timeout: Duration::from_millis(500),
        tokio_rt: None,
        stun_servers: Vec::new().into(),
    })
    .unwrap();
    let pipeline = Arc::new(Mutex::new(pipeline));

    let mut inputs = Vec::new();
    for i in 0..bench_config.decoder_count {
        let input_id = InputId(format!("input_{i}").into());
        inputs.push(input_id.clone());

        Pipeline::register_input(
            &pipeline,
            input_id,
            RegisterInputOptions {
                input_options: InputOptions::Mp4(Mp4Options {
                    should_loop: true,
                    video_decoder: bench_config.video_decoder,
                    source: Source::File(bench_config.file_path.clone()),
                }),
                queue_options: QueueInputOptions {
                    offset: Some(Duration::ZERO),
                    required: true,
                    buffer_duration: None,
                },
            },
        )
        .unwrap();
    }

    let output_id = OutputId("output".into());
    let receiver = Pipeline::register_encoded_data_output(
        &pipeline,
        output_id,
        RegisterOutputOptions {
            video: Some(OutputVideoOptions {
                end_condition: PipelineOutputEndCondition::AnyInput,
                initial: Component::Tiles(TilesComponent {
                    id: None,
                    width: Some(bench_config.output_width as f32),
                    height: Some(bench_config.output_height as f32),
                    margin: 2.0,
                    padding: 0.0,
                    children: inputs
                        .into_iter()
                        .map(|i| {
                            Component::InputStream(InputStreamComponent {
                                id: None,
                                input_id: i,
                            })
                        })
                        .collect(),
                    transition: None,
                    vertical_align: VerticalAlign::Center,
                    horizontal_align: HorizontalAlign::Center,
                    background_color: RGBAColor(128, 128, 128, 0),
                    tile_aspect_ratio: (16, 9),
                }),
            }),

            audio: None,
            output_options: EncodedDataOutputOptions {
                audio: None,
                video: Some(VideoEncoderOptions::H264(H264OutputOptions {
                    preset: bench_config.output_encoder_preset,
                    resolution: Resolution {
                        width: bench_config.output_width as usize,
                        height: bench_config.output_height as usize,
                    },
                    raw_options: Vec::new(),
                })),
            },
        },
    )
    .unwrap();

    Pipeline::start(&pipeline);

    let start_time = Instant::now();
    while Instant::now() - start_time < bench_config.warm_up_time {
        _ = receiver.recv().unwrap();
    }

    let start_time = Instant::now();
    let mut produced_frames: usize = 0;
    while Instant::now() - start_time < bench_config.measured_time {
        _ = receiver.recv().unwrap();
        produced_frames += 1;
    }

    let end_time = Instant::now();

    let framerate = produced_frames as f64 / (end_time - start_time).as_secs_f64();

    framerate * bench_config.framerate_tolerance_multiplier > bench_config.framerate as f64
}
