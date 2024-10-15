import { Frame, InputId } from '@live-compositor/browser-render';
import MP4Source from './mp4/source';
import { CompositorEventType } from 'live-compositor';
import { EventSender } from '../eventSender';
import InputSource from './source';
import { RegisterInputRequest } from '@live-compositor/core';
import { H264Decoder } from './decoder/h264Decoder';
import { VideoPayload } from './payload';
import Decoder from './decoder/decoder';

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
  private eventSender: EventSender;
  /**
   * Queue PTS of the first frame
   */
  private startPtsMs?: number;

  public constructor(id: InputId, request: RegisterInputRequest, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.eventSender = eventSender;

    // TODO(noituri): Spawn source and decoder as web workers
    if (request.type === 'mp4') {
      this.source = new MP4Source(request.url!);
    } else {
      throw new Error(`Unknown input type ${(request as any).type}`);
    }

    this.decoder = new H264Decoder({
      maxDecodedFrames: 1000,
    });

    this.source.registerCallbacks({
      onDecoderConfig: config => this.decoder.configure(config),
      onPayload: payload => this.handlePayload(payload),
    });
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

  private handlePayload(payload: VideoPayload) {
    this.decoder.enqueue(payload);

    // TODO(noituri): This does not work if buffer length > samples
    // Preferably handlePayload should handle also frames
    if (this.state == 'buffering' && this.decoder.isBufferFull()) {
      this.state = 'playing';
      this.eventSender.sendEvent({
        type: CompositorEventType.VIDEO_INPUT_PLAYING,
        inputId: this.id,
      });
    }
  }

  public getId(): InputId {
    return this.id;
  }

  public async getFrame(queuePtsMs: number): Promise<InputFrame | undefined> {
    if (this.state != 'playing') {
      return;
    }

    let frame = await this.decoder.getFrame();
    if (!frame) {
      if (this.decoder.isFinished()) {
        this.state = 'finished';
        this.eventSender.sendEvent({
          type: CompositorEventType.VIDEO_INPUT_EOS,
          inputId: this.id,
        });
      }
      return;
    }

    if (!this.startPtsMs) {
      this.startPtsMs = queuePtsMs;
    }

    while (frame && frame.ptsMs + this.startPtsMs! < queuePtsMs) {
      console.warn(`Input "${this.id}": Frame dropped. PTS ${frame.ptsMs}`);
      frame.free();
      frame = await this.decoder.getFrame();
    }

    return frame;
  }
}
