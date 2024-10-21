import {
  ApiRequest,
  CompositorManager,
  RegisterInputRequest,
  RegisterOutputRequest,
} from '@live-compositor/core';
import { Renderer, Component, ImageSpec } from '@live-compositor/browser-render';
import { Api } from 'live-compositor';
import { Path } from 'path-parser';
import { Queue, StopQueueFn } from '../queue';
import { Input } from '../input/input';
import { EventSender } from '../eventSender';
import { Framerate } from '../compositor';
import { Output } from '../output/output';

export type OnRegisterCallback = (event: object) => void;

const apiPath = new Path('/api/:type/:id/:operation');
const apiStartPath = new Path('/api/start');

class WasmInstance implements CompositorManager {
  private renderer: Renderer;
  private queue: Queue;
  private eventSender: EventSender;
  private stopQueue?: StopQueueFn;

  public constructor(props: { renderer: Renderer; framerate: Framerate }) {
    this.renderer = props.renderer;
    this.queue = new Queue(props.framerate, props.renderer);
    this.eventSender = new EventSender();
  }

  public async setupInstance(): Promise<void> {}

  public async sendRequest(request: ApiRequest): Promise<object> {
    const route = apiPath.test(request.route);
    if (!route) {
      if (apiStartPath.test(request.route)) {
        this.start();
      }
      return {};
    }

    if (route.type == 'input') {
      await this.handleInputRequest(route.id, route.operation, request.body);
    } else if (route.type === 'output') {
      await this.handleOutputRequest(route.id, route.operation, request.body);
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
    this.eventSender.setEventCallback(cb);
  }

  private start() {
    if (this.stopQueue) {
      throw 'Compositor is already running';
    }
    this.stopQueue = this.queue.start();
  }

  public stop() {
    if (this.stopQueue) {
      this.stopQueue();
      this.stopQueue = undefined;
    }
  }

  private async handleInputRequest(
    inputId: string,
    operation: string,
    body?: object
  ): Promise<void> {
    if (operation === 'register') {
      const request = body! as RegisterInputRequest;
      const input = new Input(inputId, request, this.eventSender);
      this.queue.addInput(inputId, input);
      this.renderer.registerInput(inputId);
      await input.start();
    } else if (operation === 'unregister') {
      this.queue.removeInput(inputId);
      this.renderer.unregisterInput(inputId);
    }
  }

  private async handleOutputRequest(
    outputId: string,
    operation: string,
    body?: object
  ): Promise<void> {
    if (operation === 'register') {
      const request = body! as RegisterOutputRequest;
      if (request.video) {
        const output = new Output(request);
        this.queue.addOutput(outputId, output);
        try {
          this.renderer.updateScene(
            outputId,
            request.video.resolution,
            request.video.initial.root as Component
          );
        } catch (e) {
          this.queue.removeOutput(outputId);
          throw e;
        }
      }
    } else if (operation === 'unregister') {
      this.queue.removeOutput(outputId);
      this.renderer.unregisterOutput(outputId);
    } else if (operation === 'update') {
      const scene = body! as Api.UpdateOutputRequest;
      if (!scene.video) {
        return;
      }
      const output = this.queue.getOutput(outputId);
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
