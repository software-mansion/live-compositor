use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_render::{InputId, OutputId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannels {
    Mono,
    Stereo,
}

#[derive(Debug, Clone)]
pub struct AudioSamplesSet {
    pub samples: HashMap<InputId, Vec<AudioSamplesBatch>>,
    pub start_pts: Duration,
    pub length: Duration,
}

impl AudioSamplesSet {
    pub fn end_pts(&self) -> Duration {
        self.start_pts + self.length
    }
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

#[derive(Debug, Clone)]
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

impl From<AudioChannels> for opus::Channels {
    fn from(value: AudioChannels) -> Self {
        match value {
            AudioChannels::Mono => opus::Channels::Mono,
            AudioChannels::Stereo => opus::Channels::Stereo,
        }
    }
}
