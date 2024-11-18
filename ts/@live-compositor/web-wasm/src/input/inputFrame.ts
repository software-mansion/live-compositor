import type { Frame } from '@live-compositor/browser-render';
import { FrameFormat } from '@live-compositor/browser-render';
import type { FrameWithPts } from './decoder/h264Decoder';

/**
 * Represents frame produced by decoder.
 * `InputFrame` has to be manually freed from the memory by calling `free()` method. Once freed it no longer can be used.
 * `Queue` on tick pulls `InputFrame` for each input and once render finishes, manually frees `InputFrame`s.
 */
export type InputFrame = Frame & {
  /**
   * Frees `InputFrame` from memory. `InputFrame` can not be used after `free()`.
   */
  free: () => void;
};

export async function intoInputFrame(decodedFrame: FrameWithPts): Promise<InputFrame> {
  // Safari does not support conversion to RGBA
  // Chrome does not support conversion to YUV
  const isSafari = !!(window as any).safari;
  const options = {
    format: isSafari ? 'I420' : 'RGBA',
  };

  const frame = decodedFrame.frame;
  const buffer = new Uint8ClampedArray(frame.allocationSize(options as VideoFrameCopyToOptions));
  await frame.copyTo(buffer, options as VideoFrameCopyToOptions);

  return {
    resolution: {
      width: frame.displayWidth,
      height: frame.displayHeight,
    },
    format: isSafari ? FrameFormat.YUV_BYTES : FrameFormat.RGBA_BYTES,
    data: buffer,
    free: () => frame.close(),
  };
}
