import type { Api } from 'live-compositor';

export type Resolution = Api.Resolution;
export type ImageSpec = Required<Pick<Api.ImageSpec, 'asset_type' | 'url'>>;
export type Component = Extract<
  Api.Component,
  { type: 'input_stream' | 'view' | 'rescaler' | 'image' | 'text' | 'tiles' }
>;
export type RendererId = Api.RendererId;
export type InputId = Api.InputId;
export type OutputId = string;
