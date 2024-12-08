import type { InputId } from '@live-compositor/browser-render';
import { CompositorEventType } from 'live-compositor';
import type { EventSender } from '../eventSender';
import { FrameRef } from './frame';
import { assert } from '../utils';
import InputFrameProducer, { DEFAULT_MAX_BUFFERING_SIZE } from './inputFrameProducer';
import { Queue } from '@datastructures-js/queue/src/queue';

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

export class Input {
  private id: InputId;
  private state: InputState;
  private frameProducer: InputFrameProducer;
  private frames: Queue<FrameRef>;
  private eventSender: EventSender;
  /**
   * Queue PTS of the first frame
   */
  private startPtsMs?: number;

  public constructor(id: InputId, frameProducer: InputFrameProducer, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.frameProducer = frameProducer;
    this.frames = new Queue();
    this.eventSender = eventSender;

    this.frameProducer.setMaxBufferSize(DEFAULT_MAX_BUFFERING_SIZE);
    this.frameProducer.registerCallbacks({
      onFrame: (frame) => this.frames.push(frame),
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

  // TODO(noituri): Comment this
  public async produceFrames(currentQueuePts: number): Promise<void> {
    let targetPts: number | undefined;
    if (this.startPtsMs !== undefined) {
      targetPts = this.queuePtsToInputPts(currentQueuePts);
    }

    await this.frameProducer.produce(targetPts);

    if (this.state === 'buffering') {
      this.handleBuffering();
      return;
    }
  }

  // TODO(noituri): Comment this
  public getFrameRef(currentQueuePts: number): FrameRef | undefined {
    if (this.state !== 'playing') {
      return;
    }
    if (this.startPtsMs === undefined) {
      this.startPtsMs = currentQueuePts;
    }

    this.dropOldFrames(currentQueuePts);

    let frame: FrameRef | undefined;
    if (this.frameProducer.isFinished() && this.frames.size() == 1) {
      frame = this.frames.pop();
    } else {
      frame = this.cloneLatestFrame();
    }

    if (frame) {
      return frame;
    }

    // EOS received and there will be no more frames
    if (this.frameProducer.isFinished()) {
      this.handleEos();
      return;
    }

    return undefined;
  }

  /**
   * Retrieves latest frame and increments its reference count
   */
  private cloneLatestFrame(): FrameRef | undefined {
    const frame = this.frames.front();
    if (frame) {
      frame.incrementRefCount();
      return frame;
    }

    return undefined;
  }

  /**
   * Finds frame with PTS closest to `currentQueuePts` and removes frames older than it
   */
  private dropOldFrames(currentQueuePts: number): void {
    if (this.frames.isEmpty()) {
      return;
    }

    const frames = this.frames.toArray();
    const targetPts = this.queuePtsToInputPts(currentQueuePts);

    const targetFrame = frames.reduce((prevFrame, frame) => {
      const prevPtsDiff = Math.abs(prevFrame.getPtsMs() - targetPts);
      const currPtsDiff = Math.abs(frame.getPtsMs() - targetPts);
      return prevPtsDiff < currPtsDiff ? prevFrame : frame;
    });

    for (const frame of frames) {
      if (frame.getPtsMs() < targetFrame.getPtsMs()) {
        frame.decrementRefCount();
        this.frames.pop();
      }
    }
  }

  private handleBuffering() {
    // TODO(noituri): Change this
    if (this.frames.size() < DEFAULT_MAX_BUFFERING_SIZE) {
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

    this.frameProducer.close();
  }

  private queuePtsToInputPts(queuePts: number): number {
    assert(this.startPtsMs !== undefined);
    return queuePts - this.startPtsMs;
  }
}
