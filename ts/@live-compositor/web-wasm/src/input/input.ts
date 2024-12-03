import type { InputId } from '@live-compositor/browser-render';
import { CompositorEventType } from 'live-compositor';
import type { EventSender } from '../eventSender';
import { FrameRef } from './frame';
import { assert } from '../utils';
import InputFrameProducer, { DEFAULT_MAX_BUFFERING_SIZE } from './inputFrameProducer';

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

    this.frameProducer.setMaxBufferSize(DEFAULT_MAX_BUFFERING_SIZE);
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

  public async getFrameRef(currentQueuePts: number): Promise<FrameRef | undefined> {
    if (this.state === 'buffering') {
      this.handleBuffering();
      return;
    }
    if (this.state !== 'playing') {
      return;
    }
    if (this.startPtsMs === undefined) {
      this.startPtsMs = currentQueuePts;
    }

    const inputPts = this.queuePtsToInputPts(currentQueuePts);
    this.dropOldFrames(inputPts);
    await this.frameProducer.produce(inputPts);


    let frame: FrameRef | undefined;

    if (frame) {
      return frame;
    }

    // Source received EOS & there is no more frames
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
    const frame = this.frameProducer.peekFrame();
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
    // if (this.frames.isEmpty()) {
    //   return;
    // const frames = this.frames.toArray();
    // const targetPts = this.queuePtsToInputPts(currentQueuePts);
    //
    // const targetFrame = frames.reduce((prevFrame, frame) => {
    //   const prevPtsDiff = Math.abs(prevFrame.getPtsMs() - targetPts);
    //   const currPtsDiff = Math.abs(frame.getPtsMs() - targetPts);
    //   return prevPtsDiff < currPtsDiff ? prevFrame : frame;
    // });
    //
    // for (const frame of frames) {
    //   if (frame.getPtsMs() < targetFrame.getPtsMs()) {
    //     frame.decrementRefCount();
    //     this.frames.pop();
    //   }
    // }
  }

  private handleBuffering() {
    // if (this.frames.size() < MAX_BUFFERING_SIZE) {
    //   return;
    // }

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

    // this.decoder.close();
  }

  private queuePtsToInputPts(queuePts: number): number {
    assert(this.startPtsMs !== undefined);
    return queuePts - this.startPtsMs;
  }


  // private enqueueChunks(currentQueuePts: number) {
  //   const framrate = this.source.getFramerate();
  //   assert(framrate);
  //
  //   const frameDuration = framerateToDurationMs(framrate);
  //   const targetPts = this.queuePtsToInputPts(currentQueuePts) + frameDuration * MAX_BUFFERING_SIZE;
  //
  //   let chunk = this.source.peekChunk();
  //   while (chunk && chunk.ptsMs < targetPts) {
  //     this.decoder.decode(chunk.data);
  //     this.source.nextChunk();
  //     chunk = this.source.peekChunk();
  //   }
  // }
}
