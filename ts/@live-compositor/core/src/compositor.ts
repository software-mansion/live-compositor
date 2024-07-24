import { RegisterInput, RegisterOutput } from 'live-compositor';
import { ApiClient, Api } from './api';
import Output from './output';
import { CompositorManager } from './compositorManager';
import { intoRegisterOutput } from './api/output';
import { intoRegisterInput } from './api/input';

export async function createLiveCompositor(manager: CompositorManager): Promise<LiveCompositor> {
  const compositor = new LiveCompositor(manager);
  await compositor['manager'].setupInstance();
  return compositor;
}

export class LiveCompositor {
  private manager: CompositorManager;
  private api: ApiClient;
  private outputs: Record<string, Output> = {};

  public constructor(manager: CompositorManager) {
    this.manager = manager;
    this.api = new ApiClient(this.manager);
  }

  public async registerOutput(outputId: string, request: RegisterOutput): Promise<object> {
    const output = new Output(outputId, request, this.api);
    const { video: initialVideo } = output.scene();

    const apiRequest = intoRegisterOutput(request, initialVideo);
    const result = await this.api.registerOutput(outputId, apiRequest);
    this.outputs[outputId] = output;
    return result;
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<object> {
    return this.api.registerInput(inputId, intoRegisterInput(request));
  }

  public async unregisterInput(inputId: string): Promise<object> {
    return this.api.unregisterInput(inputId);
  }

  public async registerShader(shaderId: string, request: Api.ShaderSpec): Promise<object> {
    return this.api.registerShader(shaderId, request);
  }

  public async unregisterShader(shaderId: string): Promise<object> {
    return this.api.unregisterShader(shaderId);
  }

  public async registerImage(imageId: string, request: Api.ImageSpec): Promise<object> {
    return this.api.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string): Promise<object> {
    return this.api.unregisterImage(imageId);
  }

  public async registerWebRenderer(
    instanceId: string,
    request: Api.WebRendererSpec
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
