import type {
  ApiRequest,
  CompositorManager,
  RegisterInputRequest,
  RegisterOutputRequest,
} from '@live-compositor/core';
import type { Framerate } from './compositor/compositor';
import type { WorkerEvent, WorkerMessage, WorkerResponse } from './workerApi';
import { EventSender } from './eventSender';
import { Path } from 'path-parser';
import { assert } from './utils';
import type { ImageSpec } from '@live-compositor/browser-render';
import type { Api } from 'live-compositor';
import type { Logger } from 'pino';
import { AsyncWorker } from './worker/bridge';

const apiPath = new Path('/api/:type/:id/:operation');
const apiStartPath = new Path('/api/start');

class WasmInstance implements CompositorManager {
  private eventSender: EventSender = new EventSender();
  private worker: AsyncWorker<WorkerMessage, WorkerResponse, WorkerEvent>;
  private logger: Logger;
  private framerate: Framerate;
  private wasmBundleUrl: string;

  public constructor(options: { framerate: Framerate; wasmBundleUrl: string; logger: Logger }) {
    this.logger = options.logger;
    this.framerate = options.framerate;
    this.wasmBundleUrl = options.wasmBundleUrl;

    const worker = new Worker(new URL('./worker/runWorker.js', import.meta.url), {
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
    const [msg, transferable] = await handleRequest(request);
    const response = await this.worker.postMessage(msg, transferable);
    return handleResponse(response);
  }

  public async registerFont(fontUrl: string): Promise<void> {
    await this.worker.postMessage({ type: 'registerFont', url: fontUrl });
  }

  public registerEventListener(cb: (event: unknown) => void): void {
    this.eventSender.registerEventCallback(cb);
  }

  public async terminate(): Promise<void> {
    await this.worker.postMessage({ type: 'terminate' });
    this.worker.terminate();
  }
}

function handleResponse(response: WorkerResponse): object {
  if (!response) {
    return {};
  }
  if (response.type === 'registerInput') {
    return response.body;
  }
  throw new Error('Unknown response type.');
}

async function handleRequest(request: ApiRequest): Promise<[WorkerMessage, Transferable[]]> {
  const route = apiPath.test(request.route);
  if (!route) {
    if (apiStartPath.test(request.route)) {
      return [{ type: 'start' }, []];
    }
    throw new Error('Unknown route');
  }

  if (route.type == 'input') {
    if (route.operation === 'register') {
      assert(request.body);
      return handleRegisterInputRequest(route.id, request.body as RegisterInputRequest);
    } else if (route.operation === 'unregister') {
      return [
        {
          type: 'unregisterInput',
          inputId: route.id,
        },
        [],
      ];
    }
  } else if (route.type === 'output') {
    if (route.operation === 'register') {
      assert(request.body);
      return handleRegisterOutputRequest(route.id, request.body as RegisterOutputRequest);
    } else if (route.operation === 'unregister') {
      return [
        {
          type: 'unregisterOutput',
          outputId: route.id,
        },
        [],
      ];
    } else if (route.operation === 'update') {
      return [
        {
          type: 'updateScene',
          outputId: route.id,
          output: request.body as Api.UpdateOutputRequest,
        },
        [],
      ];
    }
  } else if (route.type === 'image') {
    if (route.operation === 'register') {
      assert(request.body);
      return [
        {
          type: 'registerImage',
          imageId: route.id,
          image: request.body as ImageSpec,
        },
        [],
      ];
    } else if (route.operation === 'unregister') {
      return [
        {
          type: 'unregisterImage',
          imageId: route.id,
        },
        [],
      ];
    }
  } else if (route.type === 'shader') {
    throw new Error('Shaders are not supported');
  } else if (route.type === 'web-renderer') {
    throw new Error('Web renderers are not supported');
  }

  throw new Error('Unknown request');
}

async function handleRegisterOutputRequest(
  outputId: string,
  body: RegisterOutputRequest
): Promise<[WorkerMessage, Transferable[]]> {
  if (body.type === 'canvas') {
    const canvas = (body.video.canvas as HTMLCanvasElement).transferControlToOffscreen();
    return [
      {
        type: 'registerOutput',
        outputId: outputId,
        output: {
          type: 'canvas',
          video: {
            resolution: body.video.resolution,
            canvas,
            initial: body.video.initial,
          },
        },
      },
      [canvas],
    ];
  }
  throw new Error(`Unknown output type ${body.type}`);
}

async function handleRegisterInputRequest(
  inputId: string,
  body: RegisterInputRequest
): Promise<[WorkerMessage, Transferable[]]> {
  if (body.type === 'mp4') {
    assert(body.url);
    return [
      {
        type: 'registerInput',
        inputId,
        input: {
          type: 'mp4',
          url: body.url,
        },
      },
      [],
    ];
  } else if (body.type === 'camera') {
    const mediaStream = await navigator.mediaDevices.getUserMedia({
      audio: false,
      video: true,
    });
    const videoTrack = mediaStream.getVideoTracks()[0];
    // @ts-ignore
    const trackProcessor = new MediaStreamTrackProcessor({ track: videoTrack });
    return [
      {
        type: 'registerInput',
        inputId,
        input: {
          type: 'camera',
          stream: trackProcessor.readable,
        },
      },
      [trackProcessor.readable],
    ];
  } else if (body.type === 'screen_capture') {
    const mediaStream = await navigator.mediaDevices.getDisplayMedia({
      audio: false,
      video: {
        width: { max: 2000 },
        height: { max: 2000 },
      },
    });
    const videoTrack = mediaStream.getVideoTracks()[0];
    assert(videoTrack);
    // @ts-ignore
    const trackProcessor = new MediaStreamTrackProcessor({ track: videoTrack });
    return [
      {
        type: 'registerInput',
        inputId,
        input: {
          type: 'screen_capture',
          stream: trackProcessor.readable,
        },
      },
      [trackProcessor.readable],
    ];
  }
  throw new Error(`Unknown output type ${body.type}`);
}

export default WasmInstance;
