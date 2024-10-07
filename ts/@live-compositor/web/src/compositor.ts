import { Renderer } from '@live-compositor/browser-render';
import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import WasmInstance from './manager/wasmInstance';
import { Queue, StopQueueFn } from './queue';
import { EventSender } from './eventSender';
import { intoRegisterOutput, RegisterOutput } from './output/registerOutput';
import { Output } from './output/output';
import { intoRegisterInput, RegisterInput } from './input/registerInput';
import { Input } from './input/input';
import { RegisterImage } from './renderers';

export type LiveCompositorOptions = {
  framerate?: Framerate;
  streamFallbackTimeoutMs?: number;
};

export type Framerate = {
  num: number;
  den: number;
};

export default class LiveCompositor {
  private coreCompositor?: CoreLiveCompositor;
  private queue?: Queue;
  private renderer?: Renderer;
  private eventSender: EventSender;
  private stopQueue?: StopQueueFn;
  private options: LiveCompositorOptions;

  public constructor(options: LiveCompositorOptions) {
    this.options = options;
    this.eventSender = new EventSender();
  }

  public async init(): Promise<void> {
    this.renderer = await Renderer.create({
      streamFallbackTimeoutMs: this.options.streamFallbackTimeoutMs ?? 500,
    });
    this.queue = new Queue(this.options.framerate ?? { num: 30, den: 1 }, this.renderer!);
    this.coreCompositor = new CoreLiveCompositor(
      new WasmInstance({
        renderer: this.renderer!,
        onRegisterCallback: cb => this.eventSender.setEventCallback(cb),
      })
    );

    await this.coreCompositor!.init();
  }

  public async registerOutput(outputId: string, request: RegisterOutput): Promise<void> {
    await this.coreCompositor!.registerOutput(outputId, intoRegisterOutput(request));
    const output = new Output(request);
    this.queue!.addOutput(outputId, output);
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    await this.coreCompositor!.unregisterOutput(outputId);
    this.queue!.removeOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<void> {
    await this.coreCompositor!.registerInput(inputId, intoRegisterInput(request));

    const input = new Input(inputId, request, this.eventSender);
    this.queue!.addInput(inputId, input);
    await input.start();
  }

  public async unregisterInput(inputId: string): Promise<void> {
    await this.coreCompositor!.unregisterInput(inputId);
    this.queue!.removeInput(inputId);
  }

  public async registerImage(imageId: string, request: RegisterImage): Promise<void> {
    await this.coreCompositor!.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string): Promise<void> {
    await this.coreCompositor!.unregisterImage(imageId);
  }

  public async registerFont(fontUrl: string): Promise<void> {
    await this.renderer!.registerFont(fontUrl);
  }

  public start(): void {
    if (this.stopQueue) {
      throw 'Compositor is already running';
    }
    this.stopQueue = this.queue!.start();
  }

  public stop(): void {
    if (this.stopQueue) {
      this.stopQueue();
      this.stopQueue = undefined;
    }
  }
}
