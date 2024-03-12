use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

use compositor_render::{InputId, OutputId};

#[derive(Debug, Clone)]
pub struct AudioMixingParams {
    pub inputs: Vec<InputParams>,
    pub mixing_strategy: MixingStrategy,
}

#[derive(Debug, Clone)]
pub enum MixingStrategy {
    SumClip,
    SumScale,
}

#[derive(Debug, Clone)]
pub struct InputParams {
    pub input_id: InputId,
    // [0, 1] range of input volume
    pub volume: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannels {
    Mono,
    Stereo,
}

#[derive(Debug, Clone)]
pub struct InputSamplesSet {
    pub samples: HashMap<InputId, Vec<InputSamples>>,
    pub start_pts: Duration,
    pub end_pts: Duration,
}

#[derive(Debug)]
pub struct OutputSamplesSet(pub HashMap<OutputId, OutputSamples>);

#[derive(Clone)]
pub struct InputSamples {
    pub samples: Arc<Vec<(i16, i16)>>,
    pub start_pts: Duration,
}

#[derive(Debug)]
pub struct OutputSamples {
    pub samples: AudioSamples,
    pub start_pts: Duration,
}

#[derive(Clone)]
pub enum AudioSamples {
    Mono(Vec<i16>),
    Stereo(Vec<(i16, i16)>),
}

impl InputSamplesSet {
    pub fn duration(&self) -> Duration {
        self.end_pts.saturating_sub(self.start_pts)
    }
}

impl InputSamples {
    pub fn end_pts(&self, output_sample_rate: u32) -> Duration {
        self.start_pts
            + Duration::from_secs_f64(self.samples.len() as f64 / output_sample_rate as f64)
    }
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

impl Debug for InputSamples {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InputSamples(len={}, pts={:?})",
            self.samples.len(),
            self.start_pts
        )
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
