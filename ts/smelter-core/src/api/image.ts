import type { Api } from '../api.js';
import { _liveCompositorInternals } from 'live-compositor';

export type RegisterInputRequest = Api.RegisterInput;

export type ImageRef = _liveCompositorInternals.ImageRef;
export const imageRefIntoRawId = _liveCompositorInternals.imageRefIntoRawId;
export const parseImageRef = _liveCompositorInternals.parseImageRef;
