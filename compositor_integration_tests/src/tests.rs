mod audio_mixing;
mod muxed_video_audio;
mod required_inputs;
mod schedule_update;
mod unregistering;

#[test]
fn integretion_tests() {
    required_inputs::required_inputs().unwrap();
    audio_mixing::audio_mixing().unwrap();
    muxed_video_audio::muxed_video_audio().unwrap();
    schedule_update::schedule_update().unwrap();
    unregistering::unregistering().unwrap();
}
