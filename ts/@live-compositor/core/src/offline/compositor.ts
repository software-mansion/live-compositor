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
  private store: _liveCompositorInternals.OfflineInstanceContextStore;
  private renderStarted: boolean = false;
  /**
   * Start and end timestamp of an inputs (if known).
   */
  private inputTimestamps: number[] = [];

  public constructor(manager: CompositorManager) {
    this.manager = manager;
    this.api = new ApiClient(this.manager);
    this.store = new _liveCompositorInternals.OfflineInstanceContextStore();
  }

  public async init(): Promise<void> {
    this.checkNotStarted();
    await this.manager.setupInstance({ aheadOfTimeProcessing: true });
  }

  public async render(request: RegisterOutput, durationMs: number) {
    this.checkNotStarted();
    this.renderStarted = true;

    const output = new OfflineOutput(OFFLINE_OUTPUT_ID, request, this.api, this.store, durationMs);
    for (const inputTimestamp of this.inputTimestamps) {
      output.timeContext.addTimestamp({ timestamp: inputTimestamp });
    }
    const apiRequest = intoRegisterOutput(request, output.scene());
    await this.api.registerOutput(OFFLINE_OUTPUT_ID, apiRequest);
    await output.scheduleAllUpdates();
    // at this point all scene update requests should already be delivered
    output.outputShutdownStateStore.close();

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
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<object> {
    this.checkNotStarted();
    const result = await this.api.registerInput(inputId, intoRegisterInput(request));

    if (request.type === 'mp4' && request.loop) {
      this.store.addInput({
        inputId,
        offsetMs: request.offsetMs ?? 0,
        videoDurationMs: Infinity,
        audioDurationMs: Infinity,
      });
    } else {
      this.store.addInput({
        inputId,
        offsetMs: request.offsetMs ?? 0,
        videoDurationMs: result.video_duration_ms,
        audioDurationMs: result.audio_duration_ms,
      });
      if (request.offsetMs) {
        this.inputTimestamps.push(request.offsetMs);
      }
      if (result.video_duration_ms) {
        this.inputTimestamps.push((request.offsetMs ?? 0) + result.video_duration_ms);
      }
      if (result.audio_duration_ms) {
        this.inputTimestamps.push((request.offsetMs ?? 0) + result.audio_duration_ms);
      }
    }
    return result;
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
