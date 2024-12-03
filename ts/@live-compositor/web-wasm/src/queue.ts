import type { FrameSet, InputId, OutputId, Renderer } from '@live-compositor/browser-render';
import type { Framerate } from './compositor';
import type { Input, InputFrame } from './input/input';
import type { Output } from './output/output';

export type StopQueueFn = () => void;

export class Queue {
  private inputs: Record<InputId, Input> = {};
  private outputs: Record<OutputId, Output> = {};
  private renderer: Renderer;
  private framerate: Framerate;
  private currentPts: number;

  public constructor(framerate: Framerate, renderer: Renderer) {
    this.renderer = renderer;
    this.framerate = framerate;
    this.currentPts = 0;
  }

  public start(): StopQueueFn {
    const tickDuration = (1000 * this.framerate.den) / this.framerate.num;
    const queueInterval = setInterval(async () => {
      await this.onTick();
      this.currentPts += tickDuration;
    }, tickDuration);

    return () => clearInterval(queueInterval);
  }

  public addInput(inputId: InputId, input: Input) {
    if (this.inputs[inputId]) {
      throw `Input "${inputId}" already exists`;
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
      throw `Output "${outputId}" already exists`;
    }
    this.outputs[outputId] = output;
  }

  public removeOutput(outputId: OutputId) {
    delete this.outputs[outputId];
  }

  public getOutput(outputId: OutputId): Output | undefined {
    return this.outputs[outputId];
  }

  private async onTick() {
    const inputs = await this.getInputFrames();
    const outputs = this.renderer.render({
      ptsMs: this.currentPts,
      frames: inputs,
    });
    this.sendOutputs(outputs);

    for (const input of Object.values(inputs)) {
      input.free();
    }
  }

  private async getInputFrames(): Promise<Record<InputId, InputFrame>> {
    const pendingFrames = Object.entries(this.inputs).map(async ([inputId, input]) => [
      inputId,
      await input.getFrame(),
    ]);
    const frames = await Promise.all(pendingFrames);
    return Object.fromEntries(frames.filter(([_inputId, frame]) => !!frame));
  }

  private sendOutputs(outputs: FrameSet) {
    for (const [outputId, frame] of Object.entries(outputs.frames)) {
      const output = this.outputs[outputId];
      if (!output) {
        console.warn(`Output "${outputId}" not found`);
        continue;
      }
      void output.send(frame);
    }
  }
}
