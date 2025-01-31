import { loadWasmModule, Renderer } from '@live-compositor/browser-render';
import { Pipeline } from './pipeline';
import { pino, type Logger } from 'pino';
import type { InitOptions, WorkerMessage, WorkerResponse } from '../workerApi';
import { registerWorkerEntrypoint } from './bridge';
import { assert } from '../utils';

let instance: Pipeline | undefined;
let onMessageLogger: Logger = pino({ level: 'warn' });

async function initInstance(options: InitOptions) {
  await loadWasmModule(options.wasmBundleUrl);
  const renderer = await Renderer.create({
    streamFallbackTimeoutMs: 500,
  });
  const logger = pino({ level: options.loggerLevel }).child({ runtime: 'worker' });
  onMessageLogger = logger.child({ element: 'onMessage' });
  instance = new Pipeline({ renderer, framerate: options.framerate, logger });
}

registerWorkerEntrypoint<WorkerMessage, WorkerResponse>(
  async (request: WorkerMessage): Promise<WorkerResponse> => {
    if (request.type === 'init') {
      return await initInstance(request);
    }
    assert(instance);
    if (request.type === 'registerInput') {
      return await instance.registerInput(request.inputId, request.input);
    } else if (request.type === 'registerOutput') {
      return await instance.registerOutput(request.outputId, request.output);
    } else if (request.type === 'registerImage') {
      return await instance.registerImage(request.imageId, request.image);
    } else if (request.type === 'unregisterInput') {
      return await instance.unregisterInput(request.inputId);
    } else if (request.type === 'unregisterOutput') {
      return await instance.unregisterOutput(request.outputId);
    } else if (request.type === 'unregisterImage') {
      return await instance.unregisterImage(request.imageId);
    } else if (request.type === 'updateScene') {
      return await instance.updateScene(request.outputId, request.output);
    } else if (request.type === 'registerFont') {
      return await instance.registerFont(request.url);
    } else if (request.type === 'start') {
      return instance.start();
    } else if (request.type === 'terminate') {
      return await instance.terminate();
    } else {
      onMessageLogger.warn(request, 'Web worker received unknown message.');
    }
  }
);
