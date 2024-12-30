import type { InputId } from '@live-compositor/browser-render';
import { CompositorEventType, type EventSender } from '../eventSender';
import type { FrameRef } from './frame';
import { assert } from '../utils';
import type InputFrameProducer from './inputFrameProducer';

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

export class Input {
  private id: InputId;
  private state: InputState;
  private frameProducer: InputFrameProducer;
  private eventSender: EventSender;
  /**
   * Queue PTS of the first frame
   */
  private startPtsMs?: number;

  public constructor(id: InputId, frameProducer: InputFrameProducer, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.frameProducer = frameProducer;
    this.eventSender = eventSender;

    this.frameProducer.registerCallbacks({
      onReady: () => {
        this.state = 'playing';
        this.eventSender.sendEvent({
          type: CompositorEventType.VIDEO_INPUT_PLAYING,
          inputId: this.id,
        });
      },
    });
  }

  public async start(): Promise<void> {
    if (this.state !== 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }

    await this.frameProducer.start();
    this.state = 'buffering';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_DELIVERED,
      inputId: this.id,
    });
  }

  /**
   * Called on every queue tick. Produces frames for given `currentQueuePts` & handles EOS.
   */
  public async onQueueTick(currentQueuePts: number): Promise<void> {
    let targetPts: number | undefined;
    if (this.startPtsMs !== undefined) {
      targetPts = this.queuePtsToInputPts(currentQueuePts);
    }

    await this.frameProducer.produce(targetPts);

    if (this.state === 'playing' && this.frameProducer.isFinished()) {
      // EOS received and no more frames will be produced.
      this.state = 'finished';
      this.eventSender.sendEvent({
        type: CompositorEventType.VIDEO_INPUT_EOS,
        inputId: this.id,
      });

      this.frameProducer.close();
    }
  }

  /**
   * Retrieves reference of a frame closest to the provided `currentQueuePts`.
   */
  public getFrameRef(currentQueuePts: number): FrameRef | undefined {
    if (this.state !== 'playing') {
      return;
    }
    if (this.startPtsMs === undefined) {
      this.startPtsMs = currentQueuePts;
    }

    const framePts = this.queuePtsToInputPts(currentQueuePts);
    return this.frameProducer.getFrameRef(framePts);
  }

  private queuePtsToInputPts(queuePts: number): number {
    assert(this.startPtsMs !== undefined);
    return queuePts - this.startPtsMs;
  }
}
