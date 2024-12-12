import type { Renderers } from 'live-compositor';
import { _liveCompositorInternals, CompositorEventType } from 'live-compositor';
import { ApiClient } from '../api.js';
import type { CompositorManager } from '../compositorManager.js';
import type { RegisterOutput } from '../api/output.js';
import { intoRegisterOutput } from '../api/output.js';
import type { RegisterInput } from '../api/input.js';
import { intoRegisterInput } from '../api/input.js';
import { intoRegisterImage, intoRegisterWebRenderer } from '../api/renderer.js';
import OfflineOutput from './output.js';
import { parseEvent } from '../event.js';

/**
 * Offline rendering only supports one output, so we can just pick any value to use
 * as an output ID.
 */
const OFFLINE_OUTPUT_ID = 'offline_output';

export class OfflineCompositor {
  private manager: CompositorManager;
  private api: ApiClient;
  private store: _liveCompositorInternals.InstanceContextStore;
  private renderStarted: boolean = false;

  public constructor(manager: CompositorManager) {
    this.manager = manager;
    this.api = new ApiClient(this.manager);
    this.store = new _liveCompositorInternals.InstanceContextStore();
  }

  public async init(): Promise<void> {
    this.checkNotStarted();
    await this.manager.setupInstance({ aheadOfTimeProcessing: true });
  }

  public async render(request: RegisterOutput, durationMs: number) {
    this.checkNotStarted();
    this.renderStarted = true;

    const output = new OfflineOutput(OFFLINE_OUTPUT_ID, request, this.api, this.store, durationMs);
    const apiRequest = intoRegisterOutput(request, output.scene());
    await this.api.registerOutput(OFFLINE_OUTPUT_ID, apiRequest);
    await output.scheduleAllUpdates();

    // at this point all scene update requests should already be delivered

    await this.api.unregisterOutput(OFFLINE_OUTPUT_ID, { schedule_time_ms: durationMs });

    const renderPromise = new Promise<void>((res, _rej) => {
      this.manager.registerEventListener(rawEvent => {
        const event = parseEvent(rawEvent);
        if (
          event &&
          event.type === CompositorEventType.OUTPUT_DONE &&
          event.outputId === OFFLINE_OUTPUT_ID
        ) {
          res();
        }
      });
    });

    await this.api.start();

    await renderPromise;
    output.outputShutdownStateStore.close();
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<object> {
    this.checkNotStarted();
    return this.store.runBlocking(async updateStore => {
      const result = await this.api.registerInput(inputId, intoRegisterInput(request));
      updateStore({ type: 'add_input', input: { inputId } });
      return result;
    });
  }

  public async registerShader(
    shaderId: string,
    request: Renderers.RegisterShader
  ): Promise<object> {
    this.checkNotStarted();
    return this.api.registerShader(shaderId, request);
  }

  public async registerImage(imageId: string, request: Renderers.RegisterImage): Promise<object> {
    this.checkNotStarted();
    return this.api.registerImage(imageId, intoRegisterImage(request));
  }

  public async registerWebRenderer(
    instanceId: string,
    request: Renderers.RegisterWebRenderer
  ): Promise<object> {
    this.checkNotStarted();
    return this.api.registerWebRenderer(instanceId, intoRegisterWebRenderer(request));
  }

  private checkNotStarted() {
    if (this.renderStarted) {
      throw new Error('Render was already started.');
    }
  }
}
