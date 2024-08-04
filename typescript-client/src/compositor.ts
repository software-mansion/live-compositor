import * as ApiTypes from './api';
import Api from './api';
import Output, { RegisterOutput } from './outptut';
import { ServerManager } from './severInstance';
import ManagedInstance from './severInstance/managed';
import { omit } from './severInstance/utils';

class LiveCompositor {
  private manager: ServerManager;
  private api: Api;
  private outputs: Record<string, Output> = {};

  private constructor(manager: ServerManager) {
    this.manager = manager;
    this.api = new Api(this.manager);
  }

  public static async create(manager?: ServerManager): Promise<LiveCompositor> {
    const compositor = new LiveCompositor(
      manager ??
        new ManagedInstance({
          port: 8000,
          executablePath: process.env.LIVE_COMPOSITOR_PATH,
        })
    );
    await compositor.manager.ensureStarted();
    return compositor;
  }

  public async registerOutput(outputId: string, outputOpts: RegisterOutput): Promise<object> {
    const output = new Output(outputId, outputOpts, this.api);
    const { video, audio } = output.scene();

    const result = await this.api.registerOutput(outputId, {
      ...outputOpts,
      video: video && outputOpts.video && { ...omit(outputOpts.video, ['root']), initial: video },
      audio: audio && outputOpts.audio && { ...outputOpts.audio, initial: audio },
    });
    this.outputs[outputId] = output;
    return result;
  }

  public async registerInput(inputId: string, request: ApiTypes.RegisterInput): Promise<object> {
    return this.api.registerInput(inputId, request);
  }

  public async unregisterInput(inputId: string): Promise<object> {
    return this.api.unregisterInput(inputId);
  }

  public async registerShader(shaderId: string, request: ApiTypes.ShaderSpec): Promise<object> {
    return this.api.registerShader(shaderId, request);
  }

  public async unregisterShader(shaderId: string): Promise<object> {
    return this.api.unregisterShader(shaderId);
  }

  public async registerImage(imageId: string, request: ApiTypes.ImageSpec): Promise<object> {
    return this.api.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string): Promise<object> {
    return this.api.unregisterImage(imageId);
  }

  public async registerWebRenderer(
    instanceId: string,
    request: ApiTypes.WebRendererSpec
  ): Promise<object> {
    return this.api.registerWebRenderer(instanceId, request);
  }

  public async unregisterWebRenderer(instanceId: string): Promise<object> {
    return this.api.unregisterWebRenderer(instanceId);
  }

  public async start(): Promise<void> {
    return this.api.start();
  }
}

export default LiveCompositor;
