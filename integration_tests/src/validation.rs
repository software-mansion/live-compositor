use anyhow::Result;
use bytes::Bytes;
use std::{fmt, ops::Range, path::Path, time::Duration};
use tracing::info;

use crate::{
    audio_decoder::AudioChannels, output_dump_from_disk, save_failed_test_dumps,
    update_dump_on_disk,
};

mod audio;
mod video;

pub fn compare_video_dumps<P: AsRef<Path> + fmt::Debug>(
    snapshot_filename: P,
    actual: &Bytes,
    config: VideoValidationConfig,
) -> Result<()> {
    let expected = match output_dump_from_disk(&snapshot_filename) {
        Ok(expected) => expected,
        Err(err) => {
            return handle_error(err, snapshot_filename, actual);
        }
    };

    let VideoValidationConfig {
        validation_intervals,
        allowed_error,
        allowed_invalid_frames,
    } = config;

    if let Err(err) = video::validate(
        &expected,
        actual,
        &validation_intervals,
        allowed_error,
        allowed_invalid_frames,
    ) {
        save_failed_test_dumps(&expected, actual, &snapshot_filename);
        handle_error(err, snapshot_filename, actual)?;
    }

    Ok(())
}

pub fn compare_audio_dumps<P: AsRef<Path> + fmt::Debug>(
    snapshot_filename: P,
    actual: &Bytes,
    config: AudioValidationConfig,
) -> Result<()> {
    let expected = match output_dump_from_disk(&snapshot_filename) {
        Ok(expected) => expected,
        Err(err) => {
            return handle_error(err, snapshot_filename, actual);
        }
    };

    let AudioValidationConfig {
        sampling_intervals,
        allowed_error,
        channels,
    } = config;

    if let Err(err) = audio::validate(
        &expected,
        actual,
        &sampling_intervals,
        allowed_error,
        channels,
    ) {
        save_failed_test_dumps(&expected, actual, &snapshot_filename);
        handle_error(err, snapshot_filename, actual)?;
    }

    Ok(())
}

fn handle_error<P: AsRef<Path> + fmt::Debug>(
    err: anyhow::Error,
    snapshot_filename: P,
    actual: &Bytes,
) -> Result<()> {
    if cfg!(feature = "update_snapshots") {
        info!("Updating output dump: {snapshot_filename:?}");
        update_dump_on_disk(&snapshot_filename, actual).unwrap();
        return Ok(());
    };

    Err(err)
}

pub struct VideoValidationConfig {
    pub validation_intervals: Vec<Range<Duration>>,
    pub allowed_error: f32,
    pub allowed_invalid_frames: usize,
}

impl Default for VideoValidationConfig {
    fn default() -> Self {
        Self {
            validation_intervals: vec![Duration::from_secs(1)..Duration::from_secs(3)],
            allowed_error: 20.0,
            allowed_invalid_frames: 0,
        }
    }
}

pub struct AudioValidationConfig {
    pub sampling_intervals: Vec<Range<Duration>>,
    pub allowed_error: f32,
    pub channels: AudioChannels,
}

impl Default for AudioValidationConfig {
    fn default() -> Self {
        Self {
            sampling_intervals: vec![Duration::from_secs(0)..Duration::from_secs(1)],
            allowed_error: 4.0,
            channels: AudioChannels::Stereo,
        }
    }
}
