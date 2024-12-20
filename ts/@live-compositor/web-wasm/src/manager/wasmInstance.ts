import type {
  ApiRequest,
  CompositorManager,
  RegisterInputRequest,
  RegisterOutputRequest,
} from '@live-compositor/core';
import type { Renderer, Component, ImageSpec } from '@live-compositor/browser-render';
import type { Api } from 'live-compositor';
import { Path } from 'path-parser';
import type { StopQueueFn } from '../queue';
import { Queue } from '../queue';
import { Input } from '../input/input';
import { EventSender } from '../eventSender';
import type { Framerate } from '../compositor';
import { Output } from '../output/output';
import { producerFromRequest } from '../input/inputFrameProducer';

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
      this.handleOutputRequest(route.id, route.operation, request.body);
    } else if (route.type === 'image') {
      await this.handleImageRequest(route.id, route.operation, request.body);
    } else if (route.type === 'shader') {
      throw new Error('Shaders are not supported');
    } else if (route.type === 'web-renderer') {
      throw new Error('Web renderers are not supported');
    }

    return {};
  }

  public registerEventListener(cb: (event: unknown) => void): void {
    this.eventSender.setEventCallback(cb);
  }

  private start() {
    if (this.stopQueue) {
      throw new Error('Compositor is already running');
    }
    this.stopQueue = this.queue.start();
  }

  public stop() {
    // TODO(noituri): Clean all remaining `InputFrame`s & stop input processing
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
      await this.registerInput(inputId, body! as RegisterInputRequest);
    } else if (operation === 'unregister') {
      this.queue.removeInput(inputId);
      this.renderer.unregisterInput(inputId);
    }
  }

  private handleOutputRequest(outputId: string, operation: string, body?: object) {
    if (operation === 'register') {
      this.registerOutput(outputId, body! as RegisterOutputRequest);
    } else if (operation === 'unregister') {
      this.queue.removeOutput(outputId);
      this.renderer.unregisterOutput(outputId);
    } else if (operation === 'update') {
      this.updateScene(outputId, body! as Api.UpdateOutputRequest);
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

  private async registerInput(inputId: string, request: RegisterInputRequest): Promise<void> {
    const frameProducer = producerFromRequest(request);
    await frameProducer.init();

    const input = new Input(inputId, frameProducer, this.eventSender);
    // `addInput` will throw an exception if input already exists
    this.queue.addInput(inputId, input);
    this.renderer.registerInput(inputId);
    input.start();
  }

  private registerOutput(outputId: string, request: RegisterOutputRequest) {
    if (request.video) {
      const output = new Output(request);
      this.queue.addOutput(outputId, output);
      try {
        // `updateScene` implicitly registers the output.
        // In case of an error, the output has to be manually cleaned up from the renderer.
        this.renderer.updateScene(
          outputId,
          request.video.resolution,
          request.video.initial.root as Component
        );
      } catch (e) {
        this.queue.removeOutput(outputId);
        this.renderer.unregisterOutput(outputId);
        throw e;
      }
    }
  }

  private updateScene(outputId: string, request: Api.UpdateOutputRequest) {
    if (!request.video) {
      return;
    }
    const output = this.queue.getOutput(outputId);
    if (!output) {
      throw `Unknown output "${outputId}"`;
    }
    this.renderer.updateScene(outputId, output.resolution, request.video.root as Component);
  }
}

export default WasmInstance;
