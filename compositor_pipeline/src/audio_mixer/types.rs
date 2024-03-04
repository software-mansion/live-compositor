use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

use compositor_render::{InputId, OutputId};

#[derive(Debug, Clone)]
pub struct AudioMixingParams {
    pub inputs: Vec<InputParams>,
}

#[derive(Debug, Clone)]
pub struct InputParams {
    pub input_id: InputId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannels {
    Mono,
    Stereo,
}

#[derive(Debug, Clone)]
pub struct AudioSamplesSet {
    pub samples: HashMap<InputId, Vec<AudioSamplesBatch>>,
    pub start_pts: Duration,
    pub end_pts: Duration,
}

#[derive(Debug)]
pub struct OutputSamples(pub HashMap<OutputId, AudioSamplesBatch>);

#[derive(Debug, Clone)]
pub struct AudioSamplesBatch {
    pub samples: Arc<AudioSamples>,
    pub start_pts: Duration,
    pub sample_rate: u32,
}

impl AudioSamplesBatch {
    pub fn end_pts(&self) -> Duration {
        self.start_pts
            + Duration::from_secs_f64(self.samples.len() as f64 / self.sample_rate as f64)
    }
}

#[derive(Clone)]
pub enum AudioSamples {
    Mono(Vec<i16>),
    Stereo(Vec<(i16, i16)>),
}

impl AudioSamples {
    pub fn len(&self) -> usize {
        match self {
            AudioSamples::Mono(samples) => samples.len(),
            AudioSamples::Stereo(samples) => samples.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Debug for AudioSamples {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioSamples::Mono(samples) => write!(f, "AudioSamples::Mono(len={})", samples.len()),
            AudioSamples::Stereo(samples) => {
                write!(f, "AudioSamples::Stereo(len={})", samples.len())
            }
        }
    }
}

impl From<AudioChannels> for opus::Channels {
    fn from(value: AudioChannels) -> Self {
        match value {
            AudioChannels::Mono => opus::Channels::Mono,
            AudioChannels::Stereo => opus::Channels::Stereo,
        }
    }
}
