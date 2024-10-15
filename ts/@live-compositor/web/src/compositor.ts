import { Renderer } from '@live-compositor/browser-render';
import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import WasmInstance from './manager/wasmInstance';
import { intoRegisterOutput, RegisterOutput } from './output/registerOutput';
import { intoRegisterInput, RegisterInput } from './input/registerInput';
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
  private instance?: WasmInstance;
  private renderer?: Renderer;
  private options: LiveCompositorOptions;

  public constructor(options: LiveCompositorOptions) {
    this.options = options;
  }

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

  public async start(): Promise<void> {
    await this.coreCompositor?.start();
  }

  public stop(): void {
    this.instance?.stop();
  }
}
