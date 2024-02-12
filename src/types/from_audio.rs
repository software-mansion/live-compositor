use super::AudioComposition;
use compositor_render::scene;

impl From<AudioComposition> for scene::AudioComposition {
    fn from(value: AudioComposition) -> Self {
        Self(
            value
                .0
                .iter()
                .map(|input_id| input_id.clone().into())
                .collect(),
        )
    }
}
