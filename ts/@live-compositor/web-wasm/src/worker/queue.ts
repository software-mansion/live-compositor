import type { Frame, FrameSet, InputId, OutputId, Renderer } from '@live-compositor/browser-render';
import type { Framerate } from '../compositor/compositor';
import type { Input } from './input/input';
import type { Output } from './output/output';
import type { Interval } from '../utils';
import { framerateToDurationMs } from '../utils';
import type { Logger } from 'pino';

export type StopQueueFn = () => void;

export class Queue {
  private inputs: Record<InputId, Input> = {};
  private outputs: Record<OutputId, Output> = {};
  private renderer: Renderer;
  private framerate: Framerate;
  private currentPts: number;
  private startTimeMs?: number;
  private queueInterval?: Interval;
  private logger: Logger;

  public constructor(framerate: Framerate, renderer: Renderer, logger: Logger) {
    this.renderer = renderer;
    this.framerate = framerate;
    this.currentPts = 0;
    this.logger = logger;
  }

  public start() {
    this.logger.debug('Start queue');
    if (this.queueInterval) {
      throw new Error('Queue was already started.');
    }
    const tickDuration = framerateToDurationMs(this.framerate);
    // TODO: setInterval can drift, this implementation needs to be replaced
    this.queueInterval = setInterval(async () => {
      await this.onTick();
      this.currentPts += tickDuration;
    }, tickDuration);
    this.startTimeMs = Date.now();
    for (const input of Object.values(this.inputs)) {
      input.updateQueueStartTime(this.startTimeMs);
    }
  }

  public stop() {
    if (this.queueInterval) {
      clearTimeout(this.queueInterval);
    }
    for (const input of Object.values(this.inputs)) {
      input.close();
    }
  }

  public addInput(inputId: InputId, input: Input) {
    if (this.inputs[inputId]) {
      throw new Error(`Input "${inputId}" already exists`);
    }
    if (this.startTimeMs) {
      input.updateQueueStartTime(this.startTimeMs);
    }
    this.inputs[inputId] = input;
  }

  public removeInput(inputId: InputId) {
    delete this.inputs[inputId];
  }

  public getInput(inputId: InputId): Input | undefined {
    return this.inputs[inputId];
  }

  public addOutput(outputId: OutputId, output: Output) {
    if (this.outputs[outputId]) {
      throw new Error(`Output "${outputId}" already exists`);
    }
    this.outputs[outputId] = output;
  }

  public removeOutput(outputId: OutputId) {
    delete this.outputs[outputId];
  }

  public getOutput(outputId: OutputId): Output | undefined {
    return this.outputs[outputId];
  }

  private async onTick(): Promise<void> {
    const frames = await this.getInputFrames();
    this.logger.trace({ frames }, 'onQueueTick');

    const outputs = this.renderer.render({
      ptsMs: this.currentPts,
      frames,
    });
    this.sendOutputs(outputs);
  }

  private async getInputFrames(): Promise<Record<InputId, Frame>> {
    const frames: Array<[InputId, Frame | undefined]> = await Promise.all(
      Object.entries(this.inputs).map(async ([inputId, input]) => [
        inputId,
        await input.getFrame(this.currentPts),
      ])
    );
    const validFrames = frames.filter((entry): entry is [string, Frame] => !!entry[1]);

    return Object.fromEntries(validFrames);
  }

  private sendOutputs(outputs: FrameSet) {
    for (const [outputId, frame] of Object.entries(outputs.frames)) {
      const output = this.outputs[outputId];
      if (!output) {
        this.logger.warn(`Output "${outputId}" not found`);
        continue;
      }
      void output.send(frame);
    }
  }
}
