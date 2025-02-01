import type { Api } from '../api.js';
import { _smelterInternals } from '@swmansion/smelter';

export type RegisterInputRequest = Api.RegisterInput;

export type ImageRef = _smelterInternals.ImageRef;
export const imageRefIntoRawId = _smelterInternals.imageRefIntoRawId;
export const parseImageRef = _smelterInternals.parseImageRef;
