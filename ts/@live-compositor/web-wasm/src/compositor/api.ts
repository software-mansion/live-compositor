import type { Output } from '@live-compositor/core';
import type { Api, Renderers } from 'live-compositor';

export type RegisterImage = Required<Pick<Renderers.RegisterImage, 'assetType' | 'url'>>;

export type RegisterOutput =
  | {
      type: 'stream';
      video: {
        resolution: Api.Resolution;
      };
      // audio: boolean;
    }
  | {
      type: 'canvas';
      video: {
        canvas: HTMLCanvasElement;
        resolution: Api.Resolution;
      };
      // audio: boolean;
    }
  | {
      type: 'whip';
      /**
       * WHIP server endpoint.
       */
      endpointUrl: string;
      /**
       * Token for authenticating communication with the WHIP server.
       */
      bearerToken?: string;
      iceServers?: RTCConfiguration['iceServers'];
      video: {
        resolution: Api.Resolution;
        maxBitrate?: number;
      };
      // audio: boolean;
    };

export function intoRegisterOutputRequest(request: RegisterOutput): Output.RegisterOutput {
  if (request.type === 'stream') {
    return { ...request, type: 'web-wasm-stream' };
  } else if (request.type === 'canvas') {
    return {
      type: 'web-wasm-canvas',
      video: {
        canvas: request.video.canvas as HTMLCanvasElement,
        resolution: request.video.resolution,
      },
    };
  } else if (request.type === 'whip') {
    return { ...request, type: 'web-wasm-whip' };
  }
  throw new Error('Unknown output type');
}

export type RegisterInput =
  | { type: 'mp4'; url: string }
  | { type: 'camera' }
  | { type: 'screen_capture' };
