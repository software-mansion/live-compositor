import type { Api } from '../api.js';
import type {
  RegisterMp4Input,
  RegisterRtpInput,
  Inputs,
  _liveCompositorInternals,
} from 'live-compositor';

export type RegisterInputRequest = Api.RegisterInput;

export type InputRef = _liveCompositorInternals.InputRef;

export function inputRefIntoRawId(inputRef: InputRef): string {
  if (inputRef.type == 'global') {
    return `global:${inputRef.id}`;
  } else {
    return `output-local:${inputRef.id}:${inputRef.outputId}`;
  }
}

export function parseInputRef(rawId: string): InputRef {
  const split = rawId.split(':');
  if (split.length <= 2) {
    throw new Error('Invalid input ID.');
  } else if (split[0] === 'global') {
    return {
      type: 'global',
      id: split.slice(1).join(),
    };
  } else if (split[0] === 'output-local') {
    return {
      type: 'output-local',
      id: Number(split[1]),
      outputId: split.slice(2).join(),
    };
  } else {
    throw new Error(`Unknown input type (${split[0]}).`);
  }
}

export type RegisterInput =
  | ({ type: 'rtp_stream' } & RegisterRtpInput)
  | ({ type: 'mp4' } & RegisterMp4Input);

export function intoRegisterInput(input: RegisterInput): RegisterInputRequest {
  if (input.type === 'mp4') {
    return intoMp4RegisterInput(input);
  } else if (input.type === 'rtp_stream') {
    return intoRtpRegisterInput(input);
  } else {
    throw new Error(`Unknown input type ${(input as any).type}`);
  }
}

function intoMp4RegisterInput(input: Inputs.RegisterMp4Input): RegisterInputRequest {
  return {
    type: 'mp4',
    url: input.url,
    path: input.serverPath,
    loop: input.loop,
    required: input.required,
    offset_ms: input.offsetMs,
  };
}

function intoRtpRegisterInput(input: Inputs.RegisterRtpInput): RegisterInputRequest {
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
