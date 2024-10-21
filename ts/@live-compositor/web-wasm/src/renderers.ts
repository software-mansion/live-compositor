import { Renderers } from 'live-compositor';

export type RegisterImage = Required<Pick<Renderers.RegisterImage, 'assetType' | 'url'>>;
