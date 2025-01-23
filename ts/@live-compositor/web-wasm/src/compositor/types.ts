import type { Resolution } from '@live-compositor/browser-render';
import type { Renderers } from 'live-compositor';

export type RegisterImage = Required<Pick<Renderers.RegisterImage, 'assetType' | 'url'>>;

export type RegisterOutput = {
  type: 'canvas';
  video: {
    canvas: HTMLCanvasElement;
    resolution: Resolution;
  };
};

export type RegisterInput =
  | { type: 'mp4'; url: string }
  | { type: 'camera' }
  | { type: 'screen_capture' };
