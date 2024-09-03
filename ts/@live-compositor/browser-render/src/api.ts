import * as api from './api.generated';

export type Resolution = api.Resolution;
export type ImageSpec = Required<Pick<api.ImageSpec, 'asset_type' | 'url'>>;
export type Component = Extract<
  api.Component,
  { type: 'input_stream' | 'view' | 'rescaler' | 'image' | 'text' | 'tiles' }
>;
export type RendererId = api.RendererId;
export type InputId = api.InputId;
export type OutputId = string;
