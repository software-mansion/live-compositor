import { wasm } from './wasm';
import type * as Api from './api';
import { Mutex } from 'async-mutex';

export type RendererOptions = {
  /**
   * A timeout that defines when the compositor should switch to fallback on the input stream that stopped sending frames.
   */
  streamFallbackTimeoutMs: number;

  logger_level?: 'error' | 'warn' | 'info' | 'debug' | 'trace';
};

export type FrameSet = {
  ptsMs: number;
  frames: { [id: string]: Frame };
};

export type Frame = {
  resolution: Api.Resolution;
  format: FrameFormat;
  data: Uint8ClampedArray;
};

export enum FrameFormat {
  RGBA_BYTES = 'RGBA_BYTES',
  YUV_BYTES = 'YUV_BYTES',
}

export class Renderer {
  private renderer: wasm.LiveCompositorRenderer;
  private mutex: Mutex;

  private constructor(renderer: wasm.LiveCompositorRenderer) {
    this.renderer = renderer;
    this.mutex = new Mutex();
  }

  public static async create(options: RendererOptions): Promise<Renderer> {
    const renderer = await wasm.create_renderer({
      stream_fallback_timeout_ms: options.streamFallbackTimeoutMs,
      logger_level: options.logger_level ?? 'warn',
    });
    return new Renderer(renderer);
  }

  public async render(input: FrameSet): Promise<FrameSet> {
    return await this.mutex.runExclusive(() => {
      const frames = new Map(Object.entries(input.frames));
      const inputFrameSet = new wasm.FrameSet(input.ptsMs, frames);
      const output = this.renderer.render(inputFrameSet);
      return {
        ptsMs: output.pts_ms,
        frames: Object.fromEntries(output.frames),
      };
    });
  }

  public async updateScene(
    outputId: Api.OutputId,
    resolution: Api.Resolution,
    scene: Api.Component
  ): Promise<void> {
    await this.mutex.runExclusive(() => {
      this.renderer.update_scene(outputId, resolution, scene);
    });
  }

  public async registerInput(inputId: Api.InputId): Promise<void> {
    await this.mutex.runExclusive(() => {
      this.renderer.register_input(inputId);
    });
  }

  public async registerImage(rendererId: Api.RendererId, imageSpec: Api.ImageSpec): Promise<void> {
    await this.mutex.runExclusive(async () => {
      await this.renderer.register_image(rendererId, imageSpec);
    });
  }

  public async registerFont(fontUrl: string) {
    await this.mutex.runExclusive(async () => {
      await this.renderer.register_font(fontUrl);
    });
  }

  public async unregisterInput(inputId: Api.InputId) {
    await this.mutex.runExclusive(() => {
      this.renderer.unregister_input(inputId);
    });
  }

  public async unregisterImage(rendererId: Api.RendererId) {
    await this.mutex.runExclusive(() => {
      this.renderer.unregister_image(rendererId);
    });
  }

  public async unregisterOutput(outputId: Api.OutputId) {
    await this.mutex.runExclusive(() => {
      this.renderer.unregister_output(outputId);
    });
  }
}
