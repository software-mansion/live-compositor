import {
  _liveCompositorInternals,
  RegisterInput,
  RegisterOutput,
  Renderers,
} from 'live-compositor';
import { ApiClient } from './api';
import Output from './output';
import { CompositorManager } from './compositorManager';
import { intoRegisterOutput } from './api/output';
import { intoRegisterInput } from './api/input';
import { onCompositorEvent } from './event';
import { intoRegisterImage, intoRegisterWebRenderer } from './api/renderer';

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

  public async init(): Promise<void> {
    this.manager.registerEventListener((event: unknown) => onCompositorEvent(this.store, event));
    await this.manager.setupInstance();
  }

  public async registerOutput(outputId: string, request: RegisterOutput): Promise<object> {
    const output = new Output(outputId, request, this.api, this.store);

    const apiRequest = intoRegisterOutput(request, output.scene());
    const result = await this.api.registerOutput(outputId, apiRequest);
    this.outputs[outputId] = output;
    await output.ready();
    return result;
  }

  public async unregisterOutput(outputId: string): Promise<object> {
    this.outputs[outputId].close();
    delete this.outputs[outputId];
    return this.api.unregisterOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<object> {
    return this.store.runBlocking(async updateStore => {
      const result = await this.api.registerInput(inputId, intoRegisterInput(request));
      updateStore({ type: 'add_input', input: { inputId } });
      return result;
    });
  }

  public async unregisterInput(inputId: string): Promise<object> {
    return this.store.runBlocking(async updateStore => {
      const result = this.api.unregisterInput(inputId);
      updateStore({ type: 'remove_input', inputId });
      return result;
    });
  }

  public async registerShader(
    shaderId: string,
    request: Renderers.RegisterShader
  ): Promise<object> {
    return this.api.registerShader(shaderId, request);
  }

  public async unregisterShader(shaderId: string): Promise<object> {
    return this.api.unregisterShader(shaderId);
  }

  public async registerImage(imageId: string, request: Renderers.RegisterImage): Promise<object> {
    return this.api.registerImage(imageId, intoRegisterImage(request));
  }

  public async unregisterImage(imageId: string): Promise<object> {
    return this.api.unregisterImage(imageId);
  }

  public async registerWebRenderer(
    instanceId: string,
    request: Renderers.RegisterWebRenderer
  ): Promise<object> {
    return this.api.registerWebRenderer(instanceId, intoRegisterWebRenderer(request));
  }

  public async unregisterWebRenderer(instanceId: string): Promise<object> {
    return this.api.unregisterWebRenderer(instanceId);
  }

  public async start(): Promise<void> {
    return this.api.start();
  }
}
