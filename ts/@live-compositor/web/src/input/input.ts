import { Frame, InputId } from '@live-compositor/browser-render';
import MP4Source from './mp4/source';
import { CompositorEventType } from 'live-compositor';
import { EventSender } from '../eventSender';
import InputSource from './source';
import { RegisterInputRequest } from '@live-compositor/core';
import { H264Decoder } from './decoder/h264Decoder';
import { VideoPayload } from './payload';
import Decoder from './decoder/decoder';
import { Queue } from '@datastructures-js/queue';

/**
 * Represents frame produced by decoder. All `InputFrame`s have to be manually freed.
 */
export type InputFrame = Frame & {
  ptsMs: number;
  /**
   * Frees InputFrame from memory. InputFrame can not be used after `free()`.
   */
  free: () => void;
};

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

export class Input {
  private id: InputId;
  private source: InputSource;
  private decoder: Decoder;
  private state: InputState;
  private chunks: Queue<EncodedVideoChunk>;
  private frames: Queue<InputFrame>;
  private eventSender: EventSender;
  /**
   * Queue PTS of the first frame
   */
  private startPtsMs?: number;
  /**
   * Represents how many frames should be decoded and kept in buffer
   */
  private bufferSize: number;
  private eosReceived: boolean = false;

  public constructor(id: InputId, request: RegisterInputRequest, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.frames = new Queue();
    this.chunks = new Queue();
    this.eventSender = eventSender;
    this.bufferSize = 5;

    // TODO(noituri): Spawn source and decoder as web workers
    if (request.type === 'mp4') {
      this.source = new MP4Source(request.url!);
    } else {
      throw new Error(`Unknown input type ${(request as any).type}`);
    }
    this.decoder = new H264Decoder();

    this.source.registerCallbacks({
      onDecoderConfig: config => this.decoder.configure(config),
      onPayload: payload => this.handlePayload(payload),
    });
    this.decoder.registerCallbacks({
      onPayload: payload => this.handlePayload(payload),
    });
  }

  public getId(): InputId {
    return this.id;
  }

  public async start() {
    if (this.state !== 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }
    this.state = 'buffering';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_DELIVERED,
      inputId: this.id,
    });

    void this.source.start();
  }

  private async handlePayload(payload: VideoPayload) {
    if (payload.type === 'chunk') {
      this.chunks.push(payload.data);
      this.tryEnqueueChunks();
    } else if (payload.type === 'frame') {
      this.frames.push(payload.data);
      if (this.state == 'buffering' && this.frames.size() < this.bufferSize) {
        this.state = 'playing';
        this.eventSender.sendEvent({
          type: CompositorEventType.VIDEO_INPUT_PLAYING,
          inputId: this.id,
        });
      }
    } else if (payload.type === 'eos') {
      this.eosReceived = true;
    }
  }

  private tryEnqueueChunks() {
    while (
      this.frames.size() < this.bufferSize &&
      this.decoder.decodeQueueSize() < this.bufferSize
    ) {
      const chunk = this.chunks.pop();
      if (!chunk) {
        break;
      }

      this.decoder.enqueue(chunk);
    }
  }

  public async getFrame(currentQueuePtsMs: number): Promise<InputFrame | undefined> {
    if (this.state !== 'playing') {
      return;
    }

    this.tryEnqueueChunks();
    let frame = this.frames.pop();
    if (!frame) {
      if (this.eosReceived && this.chunks.isEmpty() && this.decoder.decodeQueueSize() == 0) {
        void this.decoder.close();
        this.state = 'finished';
        this.eventSender.sendEvent({
          type: CompositorEventType.VIDEO_INPUT_EOS,
          inputId: this.id,
        });
      }
      return;
    }

    if (!this.startPtsMs) {
      this.startPtsMs = currentQueuePtsMs;
    }

    while (frame && this.framePtsToQueuePtsMs(frame.ptsMs)! < currentQueuePtsMs) {
      console.warn(`Input "${this.id}": Frame dropped. PTS ${frame.ptsMs}`);
      frame.free();
      frame = this.frames.pop();
    }

    return frame;
  }

  private framePtsToQueuePtsMs(framePtsMs: number): number | undefined {
    if (this.startPtsMs) {
      return this.startPtsMs + framePtsMs;
    }

    return undefined;
  }
}
