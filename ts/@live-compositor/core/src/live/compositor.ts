import type { Renderers } from 'live-compositor';
import { _liveCompositorInternals } from 'live-compositor';
import { ApiClient } from '../api.js';
import Output from './output.js';
import type { CompositorManager } from '../compositorManager.js';
import type { RegisterOutput } from '../api/output.js';
import { intoRegisterOutput } from '../api/output.js';
import type { RegisterInput } from '../api/input.js';
import { intoRegisterInput } from '../api/input.js';
import { parseEvent } from '../event.js';
import { intoRegisterImage, intoRegisterWebRenderer } from '../api/renderer.js';
import { handleEvent } from './event.js';
import type { ReactElement } from 'react';
import type { Logger } from 'pino';

export class LiveCompositor {
  private manager: CompositorManager;
  private api: ApiClient;
  private store: _liveCompositorInternals.LiveInputStreamStore<string>;
  private outputs: Record<string, Output> = {};
  private startTime?: number;
  private logger: Logger;

  public constructor(manager: CompositorManager, logger: Logger) {
    this.manager = manager;
    this.api = new ApiClient(this.manager);
    this.store = new _liveCompositorInternals.LiveInputStreamStore(logger);
    this.logger = logger;
  }

  public async init(): Promise<void> {
    this.manager.registerEventListener((event: unknown) => this.handleEvent(event));
    await this.manager.setupInstance({
      aheadOfTimeProcessing: false,
      logger: this.logger.child({ element: 'connection-manager' }),
    });
  }

  public async registerOutput(
    outputId: string,
    root: ReactElement,
    request: RegisterOutput
  ): Promise<object> {
    this.logger.info({ outputId, type: request.type }, 'Register new output');
    const output = new Output(
      outputId,
      root,
      request,
      this.api,
      this.store,
      this.startTime,
      this.logger
    );

    const apiRequest = intoRegisterOutput(request, output.scene());
    const result = await this.api.registerOutput(outputId, apiRequest);
    this.outputs[outputId] = output;
    await output.ready();
    return result;
  }

  public async unregisterOutput(outputId: string): Promise<object> {
    this.logger.info({ outputId }, 'Unregister output');
    this.outputs[outputId].close();
    delete this.outputs[outputId];
    // TODO: wait for event
    return this.api.unregisterOutput(outputId, {});
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<object> {
    this.logger.info({ inputId, type: request.type }, 'Register new input');
    return this.store.runBlocking(async updateStore => {
      const inputRef = { type: 'global', id: inputId } as const;
      const result = await this.api.registerInput(inputRef, intoRegisterInput(request));
      updateStore({
        type: 'add_input',
        input: {
          inputId,
          videoDurationMs: result.video_duration_ms,
          audioDurationMs: result.audio_duration_ms,
        },
      });
      return result;
    });
  }

  public async unregisterInput(inputId: string): Promise<object> {
    this.logger.info({ inputId }, 'Unregister input');
    return this.store.runBlocking(async updateStore => {
      const inputRef = { type: 'global', id: inputId } as const;
      const result = this.api.unregisterInput(inputRef, {});
      updateStore({ type: 'remove_input', inputId });
      return result;
    });
  }

  public async registerShader(
    shaderId: string,
    request: Renderers.RegisterShader
  ): Promise<object> {
    this.logger.info({ shaderId }, 'Register shader');
    return this.api.registerShader(shaderId, request);
  }

  public async unregisterShader(shaderId: string): Promise<object> {
    this.logger.info({ shaderId }, 'Unregister shader');
    return this.api.unregisterShader(shaderId);
  }

  public async registerImage(imageId: string, request: Renderers.RegisterImage): Promise<object> {
    this.logger.info({ imageId }, 'Register image');
    return this.api.registerImage(imageId, intoRegisterImage(request));
  }

  public async unregisterImage(imageId: string): Promise<object> {
    this.logger.info({ imageId }, 'Unregister image');
    return this.api.unregisterImage(imageId);
  }

  public async registerWebRenderer(
    instanceId: string,
    request: Renderers.RegisterWebRenderer
  ): Promise<object> {
    this.logger.info({ instanceId }, 'Register web renderer');
    return this.api.registerWebRenderer(instanceId, intoRegisterWebRenderer(request));
  }

  public async unregisterWebRenderer(instanceId: string): Promise<object> {
    this.logger.info({ instanceId }, 'Unregister web renderer');
    return this.api.unregisterWebRenderer(instanceId);
  }

  public async start(): Promise<void> {
    this.logger.info('Start compositor instance.');
    const startTime = Date.now();
    await this.api.start();
    Object.values(this.outputs).forEach(output => {
      output.initClock(startTime);
    });
    this.startTime = startTime;
  }

  private handleEvent(rawEvent: unknown) {
    const event = parseEvent(rawEvent, this.logger);
    if (!event) {
      return;
    }
    this.logger.debug({ event }, 'New event received');
    handleEvent(this.store, this.outputs, event);
  }
}
