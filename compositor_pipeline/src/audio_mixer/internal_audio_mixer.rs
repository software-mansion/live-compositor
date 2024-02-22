use std::{collections::HashMap, time::Duration};

use compositor_render::{error::UpdateSceneError, OutputId};

use super::types::{Audio, AudioSamplesSet, OutputSamples};

#[derive(Debug)]
struct OutputInfo {
    audio: Audio,
    last_batch_pts: Option<Duration>,
}

#[derive(Debug)]
pub(super) struct InternalAudioMixer {
    outputs: HashMap<OutputId, OutputInfo>,
}

impl InternalAudioMixer {
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    pub fn mix_samples(&mut self, samples_set: AudioSamplesSet) -> OutputSamples {
        let get_batch = |output_info: &OutputInfo| {
            let input_id = &output_info.audio.inputs.first()?.input_id;
            samples_set.samples.get(input_id)?.iter().find(|batch| {
                let last_pts = match output_info.last_batch_pts {
                    Some(pts) => pts,
                    None => return true,
                };

                batch.start_pts > last_pts
            })
        };

        let samples = self
            .outputs
            .iter_mut()
            .filter_map(|(output_id, output_info)| {
                let batch = get_batch(output_info)?.clone();
                output_info.last_batch_pts = Some(batch.start_pts);
                Some((output_id.clone(), batch))
            })
            .collect();

        OutputSamples(samples)
    }

    pub fn register_output(&mut self, output_id: OutputId, audio: Audio) {
        self.outputs.insert(
            output_id,
            OutputInfo {
                audio,
                last_batch_pts: None,
            },
        );
    }

    pub fn unregister_output(&mut self, output_id: &OutputId) {
        self.outputs.remove(output_id);
    }

    pub fn update_output(
        &mut self,
        output_id: &OutputId,
        audio: Audio,
    ) -> Result<(), UpdateSceneError> {
        match self.outputs.get_mut(output_id) {
            Some(output_info) => {
                output_info.audio = audio;
                Ok(())
            }
            None => Err(UpdateSceneError::OutputNotRegistered(output_id.clone())),
        }
    }
}
