mod audio_mixing;
mod muxed_video_audio;
mod push_entire_input_before_start;
mod required_inputs;
mod schedule_update;
mod unregistering;

#[test]
fn integretion_tests() {
    //required_inputs::required_inputs().unwrap(); // 801x
    //audio_mixing::audio_mixing().unwrap(); // 803x
    //muxed_video_audio::muxed_video_audio().unwrap(); // 800x
    //schedule_update::schedule_update().unwrap(); // 804x
    //unregistering::unregistering().unwrap(); // 802x
    //push_entire_input_before_start::push_entire_input_before_start_tcp().unwrap(); // 805x
    //push_entire_input_before_start::push_entire_input_before_start_udp().unwrap(); // 806x

    // 807x
    push_entire_input_before_start::push_entire_input_before_start_tcp_without_offset().unwrap();
    // 808x
    //push_entire_input_before_start::push_entire_input_before_start_udp_without_offset().unwrap();
}
