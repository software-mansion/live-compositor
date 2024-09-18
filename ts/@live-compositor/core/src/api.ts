import { Api } from 'live-compositor';
import { CompositorManager } from './compositorManager';
import { RegisterOutputRequest } from './api/output';
import { RegisterInputRequest } from './api/input';

export { Api };

export type ApiRequest = {
  method: 'GET' | 'POST';
  route: string;
  body?: object;
};

export class ApiClient {
  private serverManager: CompositorManager;

  constructor(serverManager: CompositorManager) {
    this.serverManager = serverManager;
  }

  public async updateScene(outputId: string, request: Api.UpdateOutputRequest): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(outputId)}/update`,
      body: request,
    });
  }

  public async registerOutput(outptuId: string, request: RegisterOutputRequest): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(outptuId)}/register`,
      body: request,
    });
  }

  public async unregisterOutput(outptuId: string): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(outptuId)}/unregister`,
      body: {},
    });
  }

  public async registerInput(inputId: string, request: RegisterInputRequest): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/input/${encodeURIComponent(inputId)}/register`,
      body: request,
    });
  }

  public async unregisterInput(inputId: string): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/input/${encodeURIComponent(inputId)}/unregister`,
      body: {},
    });
  }

  public async registerShader(shaderId: string, request: Api.ShaderSpec): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/shader/${encodeURIComponent(shaderId)}/register`,
      body: request,
    });
  }

  public async unregisterShader(shaderId: string): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/shader/${encodeURIComponent(shaderId)}/unregister`,
      body: {},
    });
  }

  public async registerImage(imageId: string, request: Api.ImageSpec): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/image/${encodeURIComponent(imageId)}/register`,
      body: request,
    });
  }

  public async unregisterImage(imageId: string): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/image/${encodeURIComponent(imageId)}/unregister`,
      body: {},
    });
  }

  public async registerWebRenderer(
    instanceId: string,
    request: Api.WebRendererSpec
  ): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/web-renderer/${encodeURIComponent(instanceId)}/register`,
      body: request,
    });
  }

  public async unregisterWebRenderer(instanceId: string): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/web-renderer/${encodeURIComponent(instanceId)}/unregister`,
      body: {},
    });
  }

  public async start(): Promise<void> {
    await this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/start`,
      body: {},
    });
  }
}
