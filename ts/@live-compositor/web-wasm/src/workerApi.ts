import type { _liveCompositorInternals, Api } from 'live-compositor';
import type { ImageSpec, Resolution } from '@live-compositor/browser-render';
import type { Framerate } from './compositor/compositor';

export type RegisterInput =
  | {
      type: 'mp4';
      url: string;
    }
  | {
      type: 'camera';
      stream: ReadableStream;
    }
  | {
      type: 'screen_capture';
      stream: ReadableStream;
    };

export type RegisterOutput = {
  type: 'canvas';
  video: {
    canvas: OffscreenCanvas;
    resolution: Resolution;
    initial: Api.Video;
  };
};

export type InitOptions = {
  framerate: Framerate;
  wasmBundleUrl: string;
  loggerLevel: string;
};

export type WorkerMessage =
  | ({ type: 'init' } & InitOptions)
  | {
      type: 'start';
    }
  | {
      type: 'registerInput';
      inputId: string;
      input: RegisterInput;
    }
  | {
      type: 'unregisterInput';
      inputId: string;
    }
  | {
      type: 'registerOutput';
      outputId: string;
      output: RegisterOutput;
    }
  | {
      type: 'updateScene';
      outputId: string;
      output: Api.UpdateOutputRequest;
    }
  | {
      type: 'unregisterOutput';
      outputId: string;
    }
  | {
      type: 'registerImage';
      imageId: string;
      image: ImageSpec;
    }
  | {
      type: 'unregisterImage';
      imageId: string;
    }
  | {
      type: 'registerFont';
      url: string;
    }
  | {
      type: 'terminate';
    };

export type WorkerResponse = void | {
  type: 'registerInput';
  body: { video_duration_ms?: number; audio_duration_ms?: number };
};

export type WorkerEvent =
  | {
      type:
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_DELIVERED
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_DELIVERED
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_PLAYING
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_PLAYING
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_EOS
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_EOS;
      inputId: string;
    }
  | {
      type: _liveCompositorInternals.CompositorEventType.OUTPUT_DONE;
      outputId: string;
    };
