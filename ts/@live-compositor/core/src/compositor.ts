import { _liveCompositorInternals, RegisterInput, RegisterOutput } from 'live-compositor';
import { ApiClient, Api } from './api';
import Output from './output';
import { CompositorManager } from './compositorManager';
import { intoRegisterOutput } from './api/output';
import { intoRegisterInput } from './api/input';
import { onCompositorEvent } from './event';

export async function createLiveCompositor(manager: CompositorManager): Promise<LiveCompositor> {
  const compositor = new LiveCompositor(manager);
  await compositor['setupInstance']();
  return compositor;
}

export class LiveCompositor {
  private manager: CompositorManager;
  private api: ApiClient;
  private store: _liveCompositorInternals.InstanceContextStore;
  private outputs: Record<string, Output> = {};

  public constructor(manager: CompositorManager) {
    this.manager = manager;
    this.api = new ApiClient(this.manager);
    this.store = new _liveCompositorInternals.InstanceContextStore();
  }

  private async setupInstance() {
    this.manager.registerEventListener((event: unknown) => onCompositorEvent(this.store, event));
    await this.manager.setupInstance();
  }

  public async registerOutput(outputId: string, request: RegisterOutput): Promise<object> {
    const output = new Output(outputId, request, this.api, this.store);

    const apiRequest = intoRegisterOutput(request, output.scene());
    const result = await this.api.registerOutput(outputId, apiRequest);
    this.outputs[outputId] = output;
    return result;
  }

  public async unregisterOutput(outputId: string): Promise<object> {
    this.outputs[outputId].close();
    delete this.outputs[outputId];
    return this.api.unregisterOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<object> {
    const result = await this.api.registerInput(inputId, intoRegisterInput(request));
    this.store.addInput(inputId);
    return result;
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
