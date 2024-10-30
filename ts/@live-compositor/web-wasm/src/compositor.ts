import { Renderer } from '@live-compositor/browser-render';
import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import WasmInstance from './manager/wasmInstance';
import type { RegisterOutput } from './output/registerOutput';
import { intoRegisterOutput } from './output/registerOutput';
import type { RegisterInput } from './input/registerInput';
import { intoRegisterInput } from './input/registerInput';
import type { RegisterImage } from './renderers';

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
  private instance?: WasmInstance;
  private renderer?: Renderer;
  private options: LiveCompositorOptions;

  public constructor(options: LiveCompositorOptions) {
    this.options = options;
  }

  /*
   * Initializes LiveCompositor instance. It needs to be called before any resource is registered.
   * Outputs won't produce any results until `start()` is called.
   */
  public async init(): Promise<void> {
    this.renderer = await Renderer.create({
      streamFallbackTimeoutMs: this.options.streamFallbackTimeoutMs ?? 500,
    });
    this.instance = new WasmInstance({
      renderer: this.renderer!,
      framerate: this.options.framerate ?? { num: 30, den: 1 },
    });
    this.coreCompositor = new CoreLiveCompositor(this.instance!);

    await this.coreCompositor!.init();
  }

  public async registerOutput(outputId: string, request: RegisterOutput): Promise<void> {
    await this.coreCompositor!.registerOutput(outputId, intoRegisterOutput(request));
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    await this.coreCompositor!.unregisterOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<void> {
    await this.coreCompositor!.registerInput(inputId, intoRegisterInput(request));
  }

  public async unregisterInput(inputId: string): Promise<void> {
    await this.coreCompositor!.unregisterInput(inputId);
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

  /**
   * Starts processing pipeline. Any previously registered output will start producing video data.
   */
  public async start(): Promise<void> {
    await this.coreCompositor!.start();
  }

  /**
   * Stops processing pipeline.
   */
  public stop(): void {
    this.instance!.stop();
  }
}
