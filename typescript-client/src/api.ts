import { ServerManager } from './severInstance';
import * as ApiTypes from './api.generated';

export * from './api.generated';

class Api {
  private serverManager: ServerManager;

  constructor(serverManager: ServerManager) {
    this.serverManager = serverManager;
  }

  public async updateScene(
    outputId: string,
    request: ApiTypes.UpdateOutputRequest
  ): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(outputId)}/update`,
      body: request,
    });
  }

  public async registerOutput(outptuId: string, request: ApiTypes.RegisterOutput): Promise<object> {
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

  public async registerInput(inputId: string, request: ApiTypes.RegisterInput): Promise<object> {
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

  public async registerShader(shaderId: string, request: ApiTypes.ShaderSpec): Promise<object> {
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

  public async registerImage(imageId: string, request: ApiTypes.ImageSpec): Promise<object> {
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
    request: ApiTypes.WebRendererSpec
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

export default Api;
