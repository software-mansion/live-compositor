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
  framerate: Framerate;
  streamFallbackTimeoutMs: number;
};

export type Framerate = {
  num: number;
  den: number;
};

export default class LiveCompositor {
  private coreCompositor: CoreLiveCompositor;
  private queue: Queue;
  private renderer: Renderer;
  private eventSender?: EventSender;

  private constructor(renderer: Renderer, framerate: Framerate) {
    this.coreCompositor = new CoreLiveCompositor(
      new WasmInstance({
        renderer: renderer,
        onRegisterCallback: cb => {
          this.eventSender = new EventSender(cb);
        },
      })
    );
    this.queue = new Queue(framerate, renderer);
    this.renderer = renderer;
  }

  public static async create(options: LiveCompositorOptions): Promise<LiveCompositor> {
    const renderer = await Renderer.create({
      streamFallbackTimeoutMs: options.streamFallbackTimeoutMs,
    });
    const compositor = new LiveCompositor(renderer, options.framerate);
    await compositor.coreCompositor.init();
    return compositor;
  }

  public async registerOutput(outputId: string, request: RegisterOutput): Promise<void> {
    await this.coreCompositor.registerOutput(outputId, intoRegisterOutput(request));
    const output = Output.create(request);
    this.queue.addOutput(outputId, output);
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    await this.coreCompositor.unregisterOutput(outputId);
    this.queue.removeOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<void> {
    await this.coreCompositor.registerInput(inputId, intoRegisterInput(request));

    const input = Input.create(inputId, request, this.eventSender!);
    this.queue.addInput(inputId, input);
    input.start();
  }

  public async unregisterInput(inputId: string): Promise<void> {
    await this.coreCompositor.unregisterInput(inputId);
    this.queue.removeInput(inputId);
  }

  public async registerImage(imageId: string, request: RegisterImage): Promise<void> {
    await this.coreCompositor.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string): Promise<void> {
    await this.coreCompositor.unregisterImage(imageId);
  }

  public async registerFont(fontUrl: string): Promise<void> {
    await this.renderer.registerFont(fontUrl);
  }

  public start(): StopQueueFn {
    return this.queue.start();
  }
}
