import { ApiRequest, CompositorManager, RegisterOutputRequest } from '@live-compositor/core';
import { Renderer, Resolution, Component, ImageSpec } from '@live-compositor/browser-render';
import { Api } from 'live-compositor';
import { Path } from 'path-parser';

type Output = {
  resolution: Resolution;
};

export type OnRegisterCallback = (event: object) => void;

const apiPath = new Path('/api/:type/:id/:operation');

class WasmInstance implements CompositorManager {
  private renderer: Renderer;
  private outputs: Map<string, Output>;
  private onRegisterCallback: (cb: OnRegisterCallback) => void;

  public constructor(props: {
    renderer: Renderer;
    onRegisterCallback: (cb: OnRegisterCallback) => void;
  }) {
    this.renderer = props.renderer;
    this.onRegisterCallback = props.onRegisterCallback;
    this.outputs = new Map();
  }

  public async setupInstance(): Promise<void> {}

  public async sendRequest(request: ApiRequest): Promise<object> {
    const route = apiPath.test(request.route);
    if (!route) {
      return {};
    }

    if (route.type == 'input') {
      this.handleInputRequest(route.id, route.operation);
    } else if (route.type === 'output') {
      this.handleOutputRequest(route.id, route.operation, request.body);
    } else if (route.type === 'image') {
      await this.handleImageRequest(route.id, route.operation, request.body);
    } else if (route.type === 'shader') {
      throw 'Shaders are not supported';
    } else if (route.type === 'web-renderer') {
      throw 'Web renderers are not supported';
    }

    return {};
  }

  public registerEventListener(cb: (event: unknown) => void): void {
    this.onRegisterCallback(cb);
  }

  private handleInputRequest(inputId: string, operation: string): void {
    if (operation === 'register') {
      this.renderer.registerInput(inputId);
    } else if (operation === 'unregister') {
      this.renderer.unregisterInput(inputId);
    }
  }

  private handleOutputRequest(outputId: string, operation: string, body?: object): void {
    if (operation === 'register') {
      const outputInfo = body! as RegisterOutputRequest;
      if (outputInfo.video) {
        const resolution = outputInfo.video.resolution;
        this.outputs.set(outputId, { resolution: resolution });
        this.renderer.updateScene(
          outputId,
          resolution,
          outputInfo.video?.initial.root as Component
        );
      }
    } else if (operation === 'unregister') {
      this.renderer.unregisterOutput(outputId);
    } else if (operation === 'update') {
      const scene = body! as Api.UpdateOutputRequest;
      if (!scene.video) {
        return;
      }
      const output = this.outputs.get(outputId);
      if (!output) {
        throw `Unknown output "${outputId}"`;
      }
      this.renderer.updateScene(outputId, output.resolution, scene.video.root as Component);
    }
  }

  private async handleImageRequest(
    imageId: string,
    operation: string,
    body?: object
  ): Promise<void> {
    if (operation === 'register') {
      await this.renderer.registerImage(imageId, body as ImageSpec);
    } else if (operation === 'unregister') {
      this.renderer.unregisterImage(imageId);
    }
  }
}

export default WasmInstance;
