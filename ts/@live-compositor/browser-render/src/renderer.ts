import { wasm } from './wasm';
import * as api from './api';

export type RendererOptions = {
  /**
   * A timeout that defines when the compositor should switch to fallback on the input stream that stopped sending frames.
   */
  streamFallbackTimeoutMs: number;
};

export type Framerate = {
  num: number;
  den: number;
};

export type FrameSet<T> = {
  ptsMs: number;
  frames: Map<T, Frame>;
};

export type Frame = {
  resolution: api.Resolution;
  format: FrameFormat;
  data: Uint8ClampedArray;
};

export enum FrameFormat {
  RGBA_BYTES = 'RGBA_BYTES',
}

export class Renderer {
  private renderer: wasm.LiveCompositorRenderer;

  private constructor(renderer: wasm.LiveCompositorRenderer) {
    this.renderer = renderer;
  }

  public static async create(options: RendererOptions): Promise<Renderer> {
    const renderer = await wasm.create_renderer({
      stream_fallback_timeout_ms: options.streamFallbackTimeoutMs,
    });
    return new Renderer(renderer);
  }

  public render(input: FrameSet<api.InputId>): FrameSet<api.OutputId> {
    const inputFrameSet = new wasm.FrameSet(input.ptsMs, input.frames);
    const output = this.renderer.render(inputFrameSet);
    return {
      ptsMs: output.pts_ms,
      frames: output.frames,
    };
  }

  public updateScene(outputId: api.OutputId, resolution: api.Resolution, scene: api.Component) {
    this.renderer.update_scene(outputId, resolution, scene);
  }

  public registerInput(inputId: api.InputId) {
    this.renderer.register_input(inputId);
  }

  public async registerImage(rendererId: api.RendererId, imageSpec: api.ImageSpec) {
    await this.renderer.register_image(rendererId, imageSpec);
  }

  public async registerFont(fontUrl: string) {
    await this.renderer.register_font(fontUrl);
  }

  public unregisterInput(inputId: api.InputId) {
    this.renderer.unregister_input(inputId);
  }

  public unregisterImage(rendererId: api.RendererId) {
    this.renderer.unregister_image(rendererId);
  }

  public unregisterOutput(outputId: api.OutputId) {
    this.renderer.unregister_output(outputId);
  }
}
