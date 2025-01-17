import { loadWasmModule, Renderer } from '@live-compositor/browser-render';
import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import WasmInstance from './manager/wasmInstance';
import type { RegisterOutput } from './output/registerOutput';
import { intoRegisterOutput } from './output/registerOutput';
import type { RegisterInput } from './input/registerInput';
import { intoRegisterInput } from './input/registerInput';
import type { RegisterImage } from './renderers';
import type { ReactElement } from 'react';
import type { Logger } from 'pino';
import { pino } from 'pino';
import { assert } from './utils';

export type LiveCompositorOptions = {
  framerate?: Framerate;
  streamFallbackTimeoutMs?: number;
};

export type Framerate = {
  num: number;
  den: number;
};

let wasmBundleUrl: string | undefined;

/*
 * Defines url where WASM bundle is hosted. This method needs to be called before
 * first LiveCompositor instance is initiated.
 */
export function setWasmBundleUrl(url: string) {
  wasmBundleUrl = url;
}

export default class LiveCompositor {
  private coreCompositor?: CoreLiveCompositor;
  private instance?: WasmInstance;
  private renderer?: Renderer;
  private options: LiveCompositorOptions;
  private logger: Logger = pino({ level: 'warn' });

  public constructor(options: LiveCompositorOptions) {
    this.options = options;
  }

  /*
   * Initializes LiveCompositor instance. It needs to be called before any resource is registered.
   * Outputs won't produce any results until `start()` is called.
   */
  public async init(): Promise<void> {
    await ensureWasmModuleLoaded();
    this.renderer = await Renderer.create({
      streamFallbackTimeoutMs: this.options.streamFallbackTimeoutMs ?? 500,
    });
    this.instance = new WasmInstance({
      renderer: this.renderer!,
      framerate: this.options.framerate ?? { num: 30, den: 1 },
    });
    this.coreCompositor = new CoreLiveCompositor(this.instance, this.logger);

    await this.coreCompositor!.init();
  }

  public async registerOutput(
    outputId: string,
    root: ReactElement,
    request: RegisterOutput
  ): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerOutput(outputId, root, intoRegisterOutput(request));
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerInput(inputId, intoRegisterInput(request));
  }

  public async unregisterInput(inputId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterInput(inputId);
  }

  public async registerImage(imageId: string, request: RegisterImage): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterImage(imageId);
  }

  public async registerFont(fontUrl: string): Promise<void> {
    assert(this.renderer);
    await this.renderer.registerFont(fontUrl);
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
  public async terminate(): Promise<void> {
    await this.coreCompositor?.terminate();
    await this.instance?.terminate();
  }
}

const ensureWasmModuleLoaded = (() => {
  let loadedState: Promise<void> | undefined = undefined;
  return async () => {
    assert(wasmBundleUrl, 'Location of WASM bundle is not defined, call setWasmBundleUrl() first.');
    if (!loadedState) {
      loadedState = loadWasmModule(wasmBundleUrl);
    }
    await loadedState;
  };
})();
