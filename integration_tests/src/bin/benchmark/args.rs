use std::{path::PathBuf, time::Duration};

use compositor_pipeline::pipeline::{self, encoder::ffmpeg_h264};

#[derive(Debug, Clone, Copy)]
pub enum Argument {
    IterateExp,
    Maximize,
    Constant(u64),
}

impl Argument {
    pub fn as_constant(&self) -> Option<u64> {
        if let Self::Constant(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

impl std::str::FromStr for Argument {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "iterate_exp" {
            return Ok(Argument::IterateExp);
        }

        if s == "maximize" {
            return Ok(Argument::Maximize);
        }

        s.parse::<u64>()
            .map(Argument::Constant)
            .map_err(|e| format!("{e}"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DurationWrapper(pub Duration);

impl std::str::FromStr for DurationWrapper {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>()
            .map(|f| DurationWrapper(Duration::from_secs_f64(f)))
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum VideoDecoder {
    FfmpegH264,
    #[cfg(not(target_os = "macos"))]
    VulkanVideoH264,
}

impl From<VideoDecoder> for pipeline::VideoDecoder {
    fn from(value: VideoDecoder) -> Self {
        match value {
            VideoDecoder::FfmpegH264 => pipeline::VideoDecoder::FFmpegH264,
            #[cfg(not(target_os = "macos"))]
            VideoDecoder::VulkanVideoH264 => pipeline::VideoDecoder::VulkanVideoH264,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum EncoderPreset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
    Placebo,
}

impl From<EncoderPreset> for ffmpeg_h264::EncoderPreset {
    fn from(value: EncoderPreset) -> Self {
        match value {
            EncoderPreset::Ultrafast => ffmpeg_h264::EncoderPreset::Ultrafast,
            EncoderPreset::Superfast => ffmpeg_h264::EncoderPreset::Superfast,
            EncoderPreset::Veryfast => ffmpeg_h264::EncoderPreset::Veryfast,
            EncoderPreset::Faster => ffmpeg_h264::EncoderPreset::Faster,
            EncoderPreset::Fast => ffmpeg_h264::EncoderPreset::Fast,
            EncoderPreset::Medium => ffmpeg_h264::EncoderPreset::Medium,
            EncoderPreset::Slow => ffmpeg_h264::EncoderPreset::Slow,
            EncoderPreset::Slower => ffmpeg_h264::EncoderPreset::Slower,
            EncoderPreset::Veryslow => ffmpeg_h264::EncoderPreset::Veryslow,
            EncoderPreset::Placebo => ffmpeg_h264::EncoderPreset::Placebo,
        }
    }
}

/// Only one option can be set to "maximize"
#[derive(Debug, Clone, clap::Parser)]
pub struct Args {
    /// [possible values: iterate_exp, maximize or a number]
    #[arg(long)]
    pub framerate: Argument,

    /// [possible values: iterate_exp, maximize or a number]
    #[arg(long)]
    pub decoder_count: Argument,

    #[arg(long)]
    pub file_path: PathBuf,

    #[arg(long)]
    pub output_width: u32,

    #[arg(long)]
    pub output_height: u32,

    #[arg(long)]
    pub encoder_preset: EncoderPreset,

    /// warm-up time in seconds
    #[arg(long)]
    pub warm_up_time: DurationWrapper,

    /// measuring time in seconds
    #[arg(long)]
    pub measured_time: DurationWrapper,

    #[arg(long)]
    pub video_decoder: VideoDecoder,

    /// in the end of the benchmark the framerate achieved by the compositor is multiplied by this
    /// number, before comparing to the target framerate
    #[arg(long)]
    pub framerate_tolerance: f64,
}

impl Args {
    pub fn arguments(&self) -> Box<[Argument]> {
        vec![self.framerate, self.decoder_count].into_boxed_slice()
    }

    pub fn with_arguments(&self, arguments: &[Argument]) -> SingleBenchConfig {
        SingleBenchConfig {
            framerate: arguments[0].as_constant().unwrap(),
            decoder_count: arguments[1].as_constant().unwrap(),

            file_path: self.file_path.clone(),
            output_width: self.output_width,
            output_height: self.output_height,
            warm_up_time: self.warm_up_time.0,
            measured_time: self.measured_time.0,
            video_decoder: self.video_decoder.into(),
            output_encoder_preset: self.encoder_preset.into(),
            framerate_tolerance_multiplier: self.framerate_tolerance,
        }
    }
}

pub struct SingleBenchConfig {
    pub decoder_count: u64,
    pub framerate: u64,
    pub file_path: PathBuf,
    pub output_width: u32,
    pub output_height: u32,
    pub output_encoder_preset: ffmpeg_h264::EncoderPreset,
    pub warm_up_time: Duration,
    pub measured_time: Duration,
    pub video_decoder: pipeline::VideoDecoder,
    pub framerate_tolerance_multiplier: f64,
}

impl SingleBenchConfig {
    pub fn log_running_config(&self) {
        tracing::info!(
            "checking configuration: framerate: {}, decoder count: {}",
            self.framerate,
            self.decoder_count
        );
    }

    pub fn log_as_report(&self) {
        print!("{}\t", self.decoder_count);
        print!("{}\t", self.framerate);
        print!("{}\t", self.output_width);
        print!("{}\t", self.output_height);
        print!("{:?}\t", self.output_encoder_preset);
        print!("{:?}\t", self.warm_up_time);
        print!("{:?}\t", self.measured_time);
        print!("{:?}\t", self.video_decoder);
        print!("{}\t", self.framerate_tolerance_multiplier);
        println!();
    }

    pub fn log_report_header() {
        println!("dec cnt\tfps\twidth\theight\tpreset\twarmup\tmeasured\tdec\ttol")
    }
}
