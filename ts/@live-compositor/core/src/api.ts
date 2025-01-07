import { Api } from 'live-compositor';
import type { CompositorManager } from './compositorManager.js';
import type { RegisterOutputRequest } from './api/output.js';
import { inputRefIntoRawId, type InputRef, type RegisterInputRequest } from './api/input.js';

export { Api };

export type ApiRequest = {
  method: 'GET' | 'POST';
  route: string;
  body?: object;
};

export type RegisterInputResponse = {
  video_duration_ms?: number;
  audio_duration_ms?: number;
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

  public async registerOutput(outputId: string, request: RegisterOutputRequest): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(outputId)}/register`,
      body: request,
    });
  }

  public async unregisterOutput(
    outputId: string,
    body: { schedule_time_ms?: number }
  ): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(outputId)}/unregister`,
      body,
    });
  }

  public async registerInput(
    inputId: InputRef,
    request: RegisterInputRequest
  ): Promise<RegisterInputResponse> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/input/${encodeURIComponent(inputRefIntoRawId(inputId))}/register`,
      body: request,
    });
  }

  public async unregisterInput(
    inputId: InputRef,
    body: { schedule_time_ms?: number }
  ): Promise<object> {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/input/${encodeURIComponent(inputRefIntoRawId(inputId))}/unregister`,
      body,
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
    console.log('REGISTER IMAGE TEST ', request);
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
