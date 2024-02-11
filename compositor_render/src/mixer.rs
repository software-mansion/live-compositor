use std::sync::{Arc, Mutex};

use crate::{AudioSamplesSet, OutputSamples};

use self::audio_mixer::InternalAudioMixer;

mod audio_mixer;

#[derive(Debug, Clone)]
pub struct AudioMixer(Arc<Mutex<InternalAudioMixer>>);

impl AudioMixer {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(InternalAudioMixer::new())))
    }

    pub fn mix_samples(&self, samples_set: AudioSamplesSet) -> OutputSamples {
        self.0.lock().unwrap().mix_samples(samples_set)
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
