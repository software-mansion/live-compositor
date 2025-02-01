import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import type { ReactElement } from 'react';
import type { Logger } from 'pino';
import { pino } from 'pino';
import { assert } from '../utils';
import {
  type RegisterOutput,
  type RegisterInput,
  type RegisterImage,
  intoRegisterOutputRequest,
} from './api';
import WasmInstance from '../mainContext/instance';
import type { RegisterOutputResponse } from '../mainContext/output';

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
    assert(wasmBundleUrl, 'Location of WASM bundle is not defined, call setWasmBundleUrl() first.');
    this.instance = new WasmInstance({
      framerate: this.options.framerate ?? { num: 30, den: 1 },
      wasmBundleUrl,
      logger: this.logger.child({ element: 'wasmInstance' }),
    });
    this.coreCompositor = new CoreLiveCompositor(this.instance, this.logger);

    await this.coreCompositor!.init();
  }

  public async registerOutput(
    outputId: string,
    root: ReactElement,
    request: RegisterOutput
  ): Promise<{ stream?: MediaStream }> {
    assert(this.coreCompositor);
    const response = (await this.coreCompositor.registerOutput(
      outputId,
      root,
      intoRegisterOutputRequest(request)
    )) as RegisterOutputResponse | undefined;
    if (response?.type === 'web-wasm-stream' || response?.type === 'web-wasm-whip') {
      return { stream: response.stream };
    } else {
      return {};
    }
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerInput(inputId, request);
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
    assert(this.instance);
    await this.instance.registerFont(new URL(fontUrl, import.meta.url).toString());
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
  }
}
