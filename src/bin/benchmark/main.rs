use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use compositor_pipeline::{
    pipeline::{
        encoder::{
            ffmpeg_h264::{self, EncoderPreset},
            VideoEncoderOptions,
        },
        input::{
            mp4::{Mp4Options, Source},
            InputOptions,
        },
        output::EncodedDataOutputOptions,
        GraphicsContext, Options, OutputVideoOptions, PipelineOutputEndCondition,
        RegisterInputOptions, RegisterOutputOptions, VideoDecoder,
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
use log::warn;
use tracing::info;

#[derive(Debug, Clone)]
struct BenchConfig {
    framerate: u32,
    file_path: PathBuf,
    output_width: u32,
    output_height: u32,
    output_encoder_preset: ffmpeg_h264::EncoderPreset,
    allowed_late_frames: u32,
    frames_in_test: u32,
    video_decoder: VideoDecoder,
}

struct SingleBenchConfig {
    bench_config: BenchConfig,
    decoders_count: usize,
}

// range decoders, one tiles output with an encoder
// need to create an env filter

fn main() {
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

    let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    file_path.push("integration_tests");
    file_path.push("examples");
    file_path.push("assets");
    file_path.push("BigBuckBunny.mp4");

    if cfg!(debug_assertions) {
        warn!("This benchmark is running in debug mode. Make sure to run in release mode for reliable results.");
    }

    let result = run(
        ctx,
        BenchConfig {
            framerate: 24,
            file_path,
            output_width: 1920,
            output_height: 1080,
            output_encoder_preset: EncoderPreset::Fast,
            allowed_late_frames: 3,
            frames_in_test: 150,
            video_decoder: VideoDecoder::VulkanVideoH264,
        },
    );

    println!("found the maximum number of input videos for this configuration on this machine to be {result}.");
}

fn run(ctx: GraphicsContext, bench_config: BenchConfig) -> usize {
    let mut upper_bound = 1;

    info!("testing {upper_bound} inputs.");
    while run_single_test(
        ctx.clone(),
        SingleBenchConfig {
            bench_config: bench_config.clone(),
            decoders_count: upper_bound,
        },
    ) {
        upper_bound *= 2;
        info!("testing {upper_bound} inputs.");
    }

    info!("found the upper bound for binsearch as {upper_bound}.");

    let mut start = upper_bound / 2;
    let mut end = upper_bound - 1;

    while end > start {
        let midpoint = (start + end + 1) / 2;

        info!("testing {midpoint} inputs.");
        if run_single_test(
            ctx.clone(),
            SingleBenchConfig {
                bench_config: bench_config.clone(),
                decoders_count: midpoint,
            },
        ) {
            start = midpoint;
        } else {
            end = midpoint - 1;
        }

        std::thread::sleep(Duration::from_secs_f64(0.5));
    }

    return end;
}

/// true - works
/// false - too slow
fn run_single_test(ctx: GraphicsContext, bench_config: SingleBenchConfig) -> bool {
    let (pipeline, _event_loop) = Pipeline::new(Options {
        queue_options: QueueOptions {
            never_drop_output_frames: true,
            output_framerate: Framerate {
                num: bench_config.bench_config.framerate,
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
    })
    .unwrap();
    let pipeline = Arc::new(Mutex::new(pipeline));

    let mut inputs = Vec::new();
    for i in 0..bench_config.decoders_count {
        let input_id = InputId(format!("input_{i}").into());
        inputs.push(input_id.clone());

        Pipeline::register_input(
            &pipeline,
            input_id,
            RegisterInputOptions {
                input_options: InputOptions::Mp4(Mp4Options {
                    should_loop: true,
                    video_decoder: bench_config.bench_config.video_decoder,
                    source: Source::File(bench_config.bench_config.file_path.clone()),
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
    let receiver = pipeline
        .lock()
        .unwrap()
        .register_encoded_data_output(
            output_id,
            RegisterOutputOptions {
                video: Some(OutputVideoOptions {
                    end_condition: PipelineOutputEndCondition::AnyInput,
                    initial: Component::Tiles(TilesComponent {
                        id: None,
                        width: Some(bench_config.bench_config.output_width as f32),
                        height: Some(bench_config.bench_config.output_height as f32),
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
                        preset: bench_config.bench_config.output_encoder_preset,
                        resolution: Resolution {
                            width: bench_config.bench_config.output_width as usize,
                            height: bench_config.bench_config.output_height as usize,
                        },
                        raw_options: Vec::new(),
                    })),
                },
            },
        )
        .unwrap();

    Pipeline::start(&pipeline);

    let mut first_frame_timestamp = None;
    let mut first_frame_pts = None;
    let mut late_frames_count = 0;

    for _ in 0..bench_config.bench_config.frames_in_test {
        let frame = receiver.recv().unwrap();

        let pts = match frame {
            compositor_pipeline::pipeline::EncoderOutputEvent::Data(encoded_chunk) => {
                encoded_chunk.pts
            }
            compositor_pipeline::pipeline::EncoderOutputEvent::AudioEOS
            | compositor_pipeline::pipeline::EncoderOutputEvent::VideoEOS => {
                panic!("unexpected end of output")
            }
        };

        let current_timestamp = Instant::now();

        if let (Some(first_ts), Some(first_pts)) = (first_frame_timestamp, first_frame_pts) {
            let pts_difference = pts - first_pts;
            let ts_difference = current_timestamp - first_ts;

            if pts_difference < ts_difference {
                // info!(
                //     "incorrect: pts_difference: {} < ts_difference: {}",
                //     pts_difference.as_secs_f64(),
                //     ts_difference.as_secs_f64()
                // );
                late_frames_count += 1;
                if late_frames_count > bench_config.bench_config.allowed_late_frames {
                    return false;
                }
            } else {
                // info!(
                //     "correct: pts_difference: {} < ts_difference: {}",
                //     pts_difference.as_secs_f64(),
                //     ts_difference.as_secs_f64()
                // );
            }
        } else {
            first_frame_pts = Some(pts);
            first_frame_timestamp = Some(current_timestamp);
        }
    }

    return true;
}
