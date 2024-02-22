use crate::audio_mixer::types::AudioChannels;

#[derive(Debug, Clone)]
pub struct Options {
    pub sample_rate: u32,
    pub channels: AudioChannels,
}
