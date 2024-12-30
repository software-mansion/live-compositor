import type { Frame } from '@live-compositor/browser-render';
import { FrameFormat } from '@live-compositor/browser-render';
import type { FrameWithPts } from './decoder/h264Decoder';
import { assert } from '../utils';

/**
 * Represents frame produced by decoder.
 * Memory has to be manually managed by incrementing reference count on `FrameRef` copy and decrementing it once it's no longer used
 * `Input` manages memory in `getFrameRef()`
 * `Queue` on tick pulls `FrameRef` for each input and once render finishes, decrements the ref count
 */
export class FrameRef {
  private frame: FrameWithPts;
  private refCount: number;
  private downloadedFrame?: Frame;

  public constructor(frame: FrameWithPts) {
    this.frame = frame;
    this.refCount = 1;
  }

  /**
   *  Increments reference count. Should be called every time the reference is copied.
   */
  public incrementRefCount(): void {
    assert(this.refCount > 0);
    this.refCount++;
  }

  /**
   * Decrements reference count. If reference count reaches 0, `FrameWithPts` is freed from the memory.
   * It's unsafe to use the returned frame after `decrementRefCount()` call.
   * Should be used after we're sure we no longer need the frame.
   */
  public decrementRefCount(): void {
    assert(this.refCount > 0);

    this.refCount--;
    if (this.refCount === 0) {
      this.frame.frame.close();
    }
  }

  /**
   * Returns underlying frame. Fails if frame was freed from memory.
   */
  public async getFrame(): Promise<Frame> {
    assert(this.refCount > 0);

    if (!this.downloadedFrame) {
      this.downloadedFrame = await downloadFrame(this.frame);
    }
    return this.downloadedFrame;
  }

  public getPtsMs(): number {
    return this.frame.ptsMs;
  }
}

export class NonCopyableFrameRef extends FrameRef {
  public constructor(frame: FrameWithPts) {
    super(frame);
  }

  public incrementRefCount(): void {
    throw new Error('Reference count of `NonCopyableFrameRef` cannot be incremented');
  }
}

async function downloadFrame(frameWithPts: FrameWithPts): Promise<Frame> {
  // Safari does not support conversion to RGBA
  // Chrome does not support conversion to YUV
  const isSafari = !!(window as any).safari;
  const options = {
    format: isSafari ? 'I420' : 'RGBA',
  };

  const frame = frameWithPts.frame;
  const buffer = new Uint8ClampedArray(frame.allocationSize(options as VideoFrameCopyToOptions));
  await frame.copyTo(buffer, options as VideoFrameCopyToOptions);

  return {
    resolution: {
      width: frame.displayWidth,
      height: frame.displayHeight,
    },
    format: isSafari ? FrameFormat.YUV_BYTES : FrameFormat.RGBA_BYTES,
    data: buffer,
  };
}
