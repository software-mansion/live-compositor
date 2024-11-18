import type { InputId } from '@live-compositor/browser-render';
import { CompositorEventType } from 'live-compositor';
import type { EventSender } from '../eventSender';
import type InputSource from './source';
import { Queue } from '@datastructures-js/queue';
import type { FrameWithPts } from './decoder/h264Decoder';
import { H264Decoder } from './decoder/h264Decoder';
import type { InputFrame } from './inputFrame';
import { intoInputFrame } from './inputFrame';

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

const MAX_BUFFERING_SIZE = 3;

export class Input {
  private id: InputId;
  private state: InputState;
  private source: InputSource;
  private decoder: H264Decoder;
  private eventSender: EventSender;
  private frames: Queue<FrameWithPts>;
  /**
   * Queue PTS of the first frame
   */
  private startPtsMs?: number;

  public constructor(id: InputId, source: InputSource, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.source = source;
    this.eventSender = eventSender;
    this.frames = new Queue();
    this.decoder = new H264Decoder({
      onFrame: frame => this.frames.push(frame),
    });

    this.source.registerCallbacks({
      onDecoderConfig: config => this.decoder.configure(config),
    });
  }

  public start() {
    if (this.state !== 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }

    this.source.start();
    this.state = 'buffering';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_DELIVERED,
      inputId: this.id,
    });
  }

  public async getFrame(currentQueuePts: number): Promise<InputFrame | undefined> {
    this.enqueueChunks();
    if (this.state === 'buffering') {
      this.handleBuffering();
    }
    if (this.state !== 'playing') {
      return undefined;
    }

    await this.dropOldFrames(currentQueuePts);

    const frame = this.frames.pop();
    if (!frame) {
      if (!this.source.isFinished()) {
        return undefined;
      }

      if (this.decoder.decodeQueueSize() === 0) {
        // Source and Decoder finished their jobs
        this.handleEos();
        return undefined;
      }

      await this.decoder.flush();
      const frame = this.frames.pop();
      return frame && intoInputFrame(frame);
    }

    return intoInputFrame(frame);
  }

  /**
   * Removes frames older than provided `currentQueuePts`
   */
  private async dropOldFrames(currentQueuePts: number): Promise<void> {
    if (!this.startPtsMs) {
      this.startPtsMs = currentQueuePts;
    }

    let frame = this.frames.front();
    while (frame && this.framePtsToQueuePts(frame.ptsMs)! < currentQueuePts) {
      console.warn(`Input "${this.id}": Frame dropped. PTS ${frame.ptsMs}`);
      frame.frame.close();
      this.enqueueChunks();
      this.frames.pop();

      frame = this.frames.front();
    }
  }

  private framePtsToQueuePts(framePtsMs: number): number | undefined {
    if (this.startPtsMs) {
      return this.startPtsMs + framePtsMs;
    }

    return undefined;
  }

  private handleBuffering() {
    if (this.frames.size() < MAX_BUFFERING_SIZE) {
      return;
    }

    this.state = 'playing';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_PLAYING,
      inputId: this.id,
    });
  }

  private handleEos() {
    this.state = 'finished';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_EOS,
      inputId: this.id,
    });

    this.decoder.close();
  }

  private enqueueChunks() {
    while (
      this.frames.size() < MAX_BUFFERING_SIZE &&
      this.decoder.decodeQueueSize() < MAX_BUFFERING_SIZE
    ) {
      const chunk = this.source.nextChunk();
      if (!chunk) {
        break;
      }
      this.decoder.decode(chunk);
    }
  }
}
