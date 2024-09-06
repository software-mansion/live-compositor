import { RegisterOutput, Api, Outputs } from 'live-compositor';

export function intoRegisterOutput(
  output: RegisterOutput,
  initial: { video?: Api.Video; audio?: Api.Audio }
): Api.RegisterOutput {
  if (output.type === 'rtp_stream') {
    return intoRegisterRtpOutput(output, initial);
  } else if (output.type === 'mp4') {
    return intoRegisterMp4Output(output, initial);
  } else {
    throw new Error(`Unknown input type ${(output as any).type}`);
  }
}

function intoRegisterRtpOutput(
  output: Outputs.RegisterRtpOutput,
  initial: { video?: Api.Video; audio?: Api.Audio }
): Api.RegisterOutput {
  return {
    type: 'rtp_stream',
    port: output.port,
    ip: output.ip,
    transport_protocol: output.transportProtocol,
    video: output.video && initial.video && intoOutputVideoOptions(output.video, initial.video),
    audio: output.audio && initial.audio && intoOutputRtpAudioOptions(output.audio, initial.audio),
  };
}

function intoRegisterMp4Output(
  output: Outputs.RegisterMp4Output,
  initial: { video?: Api.Video; audio?: Api.Audio }
): Api.RegisterOutput {
  return {
    type: 'mp4',
    path: output.serverPath,
    video: output.video && initial.video && intoOutputVideoOptions(output.video, initial.video),
    audio: output.audio && initial.audio && intoOutputMp4AudioOptions(output.audio, initial.audio),
  };
}

function intoOutputVideoOptions(
  video: Outputs.OutputRtpVideoOptions | Outputs.OutputMp4VideoOptions,
  initial: Api.Video
): Api.OutputVideoOptions {
  return {
    resolution: video.resolution,
    send_eos_when: video.sendEosWhen && intoOutputEosCondition(video.sendEosWhen),
    encoder: intoVideoEncoderOptions(video.encoder),
    initial,
  };
}

function intoVideoEncoderOptions(
  encoder: Outputs.RtpVideoEncoderOptions | Outputs.Mp4VideoEncoderOptions
): Api.VideoEncoderOptions {
  return {
    type: 'ffmpeg_h264',
    preset: encoder.preset,
    ffmpeg_options: encoder.ffmpegOptions,
  };
}

function intoOutputRtpAudioOptions(
  audio: Outputs.OutputRtpAudioOptions,
  initial: Api.Audio
): Api.OutputRtpAudioOptions {
  return {
    send_eos_when: audio.sendEosWhen && intoOutputEosCondition(audio.sendEosWhen),
    encoder: intoRtpAudioEncoderOptions(audio.encoder),
    initial,
  };
}

function intoOutputMp4AudioOptions(
  audio: Outputs.OutputMp4AudioOptions,
  initial: Api.Audio
): Api.OutputMp4AudioOptions {
  return {
    send_eos_when: audio.sendEosWhen && intoOutputEosCondition(audio.sendEosWhen),
    encoder: intoMp4AudioEncoderOptions(audio.encoder),
    initial,
  };
}

function intoRtpAudioEncoderOptions(
  encoder: Outputs.RtpAudioEncoderOptions
): Api.RtpAudioEncoderOptions {
  return {
    type: 'opus',
    preset: encoder.preset,
    channels: encoder.channels,
  };
}

function intoMp4AudioEncoderOptions(
  encoder: Outputs.Mp4AudioEncoderOptions
): Api.Mp4AudioEncoderOptions {
  return {
    type: 'aac',
    channels: encoder.channels,
  };
}

export function intoAudioInputsConfiguration(audio: Outputs.AudioInputsConfiguration): Api.Audio {
  return {
    inputs: audio.inputs.map(input => ({
      input_id: input.inputId,
      volume: input.volume,
    })),
  };
}

function intoOutputEosCondition(condition: Outputs.OutputEndCondition): Api.OutputEndCondition {
  if ('anyOf' in condition) {
    return { any_of: condition.anyOf };
  } else if ('allOf' in condition) {
    return { all_of: condition.allOf };
  } else if ('allInputs' in condition) {
    return { all_inputs: condition.allInputs };
  } else if ('anyInput' in condition) {
    return { any_input: condition.anyInput };
  } else {
    throw new Error('Invalid "send_eos_when" value.');
  }
}