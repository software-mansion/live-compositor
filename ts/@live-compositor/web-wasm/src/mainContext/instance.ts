import type {
  ApiRequest,
  CompositorManager,
  Input as CoreInput,
  Output as CoreOutput,
  MultipartRequest,
} from '@live-compositor/core';
import type { Framerate } from '../compositor/compositor';
import type { WorkerEvent, WorkerMessage, WorkerResponse } from '../workerApi';
import { EventSender } from '../eventSender';
import { Path } from 'path-parser';
import { assert } from '../utils';
import type { ImageSpec } from '@live-compositor/browser-render';
import type { Api } from 'live-compositor';
import type { Logger } from 'pino';
import { AsyncWorker } from '../workerContext/bridge';
import { handleRegisterOutputRequest, type Output } from './output';
import type { Input } from './input';
import { handleRegisterInputRequest } from './input';

const apiPath = new Path('/api/:type/:id/:operation');
const apiStartPath = new Path('/api/start');

class WasmInstance implements CompositorManager {
  private eventSender: EventSender = new EventSender();
  private worker: AsyncWorker<WorkerMessage, WorkerResponse, WorkerEvent>;
  private logger: Logger;
  private framerate: Framerate;
  private wasmBundleUrl: string;
  private outputs: Record<string, Output> = {};
  private inputs: Record<string, Input> = {};

  public constructor(options: { framerate: Framerate; wasmBundleUrl: string; logger: Logger }) {
    this.logger = options.logger;
    this.framerate = options.framerate;
    this.wasmBundleUrl = options.wasmBundleUrl;

    const worker = new Worker(new URL('../workerContext/runWorker.js', import.meta.url), {
      type: 'module',
    });
    const onEvent = (event: WorkerEvent) => {
      if (EventSender.isExternalEvent(event)) {
        this.eventSender.sendEvent(event);
        return;
      }
      throw new Error(`Unknown event received. ${JSON.stringify(event)}`);
    };
    this.worker = new AsyncWorker(worker, onEvent, this.logger);
  }

  public async setupInstance(): Promise<void> {
    await this.worker.postMessage({
      type: 'init',
      framerate: this.framerate,
      wasmBundleUrl: this.wasmBundleUrl,
      loggerLevel: this.logger.level,
    });
    this.logger.debug('WASM instance initialized');
  }

  public async sendRequest(request: ApiRequest): Promise<object> {
    return await this.handleRequest(request);
  }

  sendMultipartRequest(_request: MultipartRequest): Promise<object> {
    throw new Error('Method sendMultipartRequest not implemented for web-wasm.');
  }

  public async registerFont(fontUrl: string): Promise<void> {
    await this.worker.postMessage({ type: 'registerFont', url: fontUrl });
  }

  public registerEventListener(cb: (event: unknown) => void): void {
    this.eventSender.registerEventCallback(cb);
  }

  public async terminate(): Promise<void> {
    await Promise.all(Object.values(this.outputs).map(output => output.terminate()));
    await Promise.all(Object.values(this.inputs).map(input => input.terminate()));
    await this.worker.postMessage({ type: 'terminate' });
    this.worker.terminate();
  }

  private async handleRequest(request: ApiRequest): Promise<any> {
    const route = apiPath.test(request.route);
    if (!route) {
      if (apiStartPath.test(request.route)) {
        await this.worker.postMessage({ type: 'start' });
        return;
      }
      throw new Error('Unknown route');
    }

    if (route.type == 'input') {
      if (route.operation === 'register') {
        assert(request.body);
        const { input, workerMessage } = await handleRegisterInputRequest(
          route.id,
          request.body as CoreInput.RegisterInputRequest
        );
        let result;
        try {
          result = await this.worker.postMessage(workerMessage[0], workerMessage[1]);
        } catch (err: any) {
          input.terminate().catch(err => {
            this.logger.warn({ err, outputId: route.id }, 'Failed to terminate input');
          });
          throw err;
        }
        this.inputs[route.id] = input;
        assert(result?.type === 'registerInput');
        return result?.body;
      } else if (route.operation === 'unregister') {
        const input = this.inputs[route.id];
        if (input) {
          delete this.inputs[route.id];
          await input.terminate();
        }
        return await this.worker.postMessage({
          type: 'unregisterInput',
          inputId: route.id,
        });
      }
    } else if (route.type === 'output') {
      if (route.operation === 'register') {
        assert(request.body);
        const { output, result, workerMessage } = await handleRegisterOutputRequest(
          route.id,
          request.body as CoreOutput.RegisterOutputRequest,
          this.logger.child({ outputId: route.id }),
          this.framerate
        );
        try {
          await this.worker.postMessage(workerMessage[0], workerMessage[1]);
        } catch (err: any) {
          output.terminate().catch(err => {
            this.logger.warn({ err, outputId: route.id }, 'Failed to terminate output');
          });
          throw err;
        }
        this.outputs[route.id] = output;
        return result;
      } else if (route.operation === 'unregister') {
        const output = this.inputs[route.id];
        if (output) {
          delete this.outputs[route.id];
          await output.terminate();
        }
        return await this.worker.postMessage({
          type: 'unregisterOutput',
          outputId: route.id,
        });
      } else if (route.operation === 'update') {
        return await this.worker.postMessage({
          type: 'updateScene',
          outputId: route.id,
          output: request.body as Api.UpdateOutputRequest,
        });
      }
    } else if (route.type === 'image') {
      if (route.operation === 'register') {
        assert(request.body);
        return await this.worker.postMessage({
          type: 'registerImage',
          imageId: route.id,
          image: request.body as ImageSpec,
        });
      } else if (route.operation === 'unregister') {
        return await this.worker.postMessage({
          type: 'unregisterImage',
          imageId: route.id,
        });
      }
    } else if (route.type === 'shader') {
      throw new Error('Shaders are not supported');
    } else if (route.type === 'web-renderer') {
      throw new Error('Web renderers are not supported');
    }

    throw new Error('Unknown request');
  }
}

export default WasmInstance;
