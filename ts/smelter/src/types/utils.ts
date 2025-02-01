import { imageAssetTypes, type ImageAssetType } from './registerRenderer.js';

export function isValidImageType(type: any): type is ImageAssetType {
  return imageAssetTypes.includes(type);
}
