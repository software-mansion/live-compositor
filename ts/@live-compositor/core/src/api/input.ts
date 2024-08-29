import { Api } from '../api';
import { RegisterInput, Inputs } from 'live-compositor';

export function intoRegisterInput(input: RegisterInput): Api.RegisterInput {
  if (input.type === 'mp4') {
    return intoMp4RegisterInput(input);
  } else if (input.type === 'rtp_stream') {
    return intoRtpRegisterInput(input);
  } else {
    throw new Error(`Unknown input type ${(input as any).type}`);
  }
}

function intoMp4RegisterInput(input: Inputs.RegisterMp4Input): Api.RegisterInput {
  return {
    type: 'mp4',
    url: input.url,
    path: input.serverPath,
    required: input.required,
    offset_ms: input.offsetMs,
  };
}

function intoRtpRegisterInput(input: Inputs.RegisterRtpInput): Api.RegisterInput {
  return {
    type: 'rtp_stream',
    port: input.port,
    transport_protocol: input.transportProtocol,
    video: input.video,
    audio: input.audio && intoInputAudio(input.audio),
    required: input.required,
    offset_ms: input.offsetMs,
  };
}

function intoInputAudio(audio: Inputs.InputRtpAudioOptions): Api.InputRtpAudioOptions {
  if (audio.decoder === 'opus') {
    return {
      decoder: 'opus',
      forward_error_correction: audio.forwardErrorCorrection,
    };
  } else if (audio.decoder === 'aac') {
    return {
      decoder: 'aac',
      audio_specific_config: audio.audioSpecificConfig,
      rtp_mode: audio.rtpMode,
    };
  } else {
    throw new Error(`Unknown audio decoder type: ${(audio as any).decoder}`);
  }
}
