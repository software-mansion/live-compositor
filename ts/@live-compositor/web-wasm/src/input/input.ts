import type { InputId } from '@live-compositor/browser-render';
import { CompositorEventType } from 'live-compositor';
import type { EventSender } from '../eventSender';
import { FrameRef } from './frame';
import { assert } from '../utils';
import InputFrameProducer from './inputFrameProducer';

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
      }
    })
  }

  public start() {
    if (this.state !== 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }

    this.frameProducer.start();
    this.state = 'buffering';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_DELIVERED,
      inputId: this.id,
    });
  }

  /**
   * On every queue tick, produces frames and handles input state changes.
   */
  public async onQueueTick(currentQueuePts: number): Promise<void> {
    let targetPts: number | undefined;
    if (this.startPtsMs !== undefined) {
      targetPts = this.queuePtsToInputPts(currentQueuePts);
    }

    await this.frameProducer.produce(targetPts);

    if (this.state === 'playing') {
      this.handlePlayingState();
    }
  }

  private handlePlayingState() {
    if (!this.frameProducer.isFinished()) {
      return;
    }

    // EOS received and no frames left in the buffer
    this.state = 'finished';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_EOS,
      inputId: this.id,
    });

    this.frameProducer.close();
  }

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
