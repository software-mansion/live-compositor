import type { RegisterMp4Input } from 'live-compositor';
import { _liveCompositorInternals } from 'live-compositor';
import type { ReactElement } from 'react';
import { createElement } from 'react';
import type { ApiClient, Api } from '../api.js';
import Renderer from '../renderer.js';
import type { RegisterOutput } from '../api/output.js';
import { intoAudioInputsConfiguration } from '../api/output.js';
import { sleep } from '../utils.js';
import { OFFLINE_OUTPUT_ID } from './compositor.js';
import { OutputRootComponent, OutputShutdownStateStore } from '../rootComponent.js';

type AudioContext = _liveCompositorInternals.AudioContext;
type OfflineTimeContext = _liveCompositorInternals.OfflineTimeContext;
type OfflineInputStreamStore<Id> = _liveCompositorInternals.OfflineInputStreamStore<Id>;
type CompositorOutputContext = _liveCompositorInternals.CompositorOutputContext;
type ChildrenLifetimeContext = _liveCompositorInternals.ChildrenLifetimeContext;

class OfflineOutput {
  api: ApiClient;
  outputId: string;
  audioContext: AudioContext;
  timeContext: OfflineTimeContext;
  childrenLifetimeContext: ChildrenLifetimeContext;
  internalInputStreamStore: OfflineInputStreamStore<number>;
  outputShutdownStateStore: OutputShutdownStateStore;
  durationMs?: number;
  updateTracker?: UpdateTracker;

  supportsAudio: boolean;
  supportsVideo: boolean;

  renderer: Renderer;

  constructor(
    root: ReactElement,
    registerRequest: RegisterOutput,
    api: ApiClient,
    store: OfflineInputStreamStore<string>,
    durationMs?: number
  ) {
    this.api = api;
    this.outputId = OFFLINE_OUTPUT_ID;
    this.outputShutdownStateStore = new OutputShutdownStateStore();
    this.durationMs = durationMs;

    this.supportsAudio = 'audio' in registerRequest && !!registerRequest.audio;
    this.supportsVideo = 'video' in registerRequest && !!registerRequest.video;

    const onUpdate = () => this.updateTracker?.onUpdate();
    this.audioContext = new _liveCompositorInternals.AudioContext(onUpdate);
    this.internalInputStreamStore = new _liveCompositorInternals.OfflineInputStreamStore();
    this.timeContext = new _liveCompositorInternals.OfflineTimeContext(
      onUpdate,
      (timestamp: number) => {
        store.setCurrentTimestamp(timestamp);
        this.internalInputStreamStore.setCurrentTimestamp(timestamp);
      }
    );
    this.childrenLifetimeContext = new _liveCompositorInternals.ChildrenLifetimeContext(() => {});

    const rootElement = createElement(OutputRootComponent, {
      outputContext: new OutputContext(this, this.outputId, store),
      outputRoot: root,
      outputShutdownStateStore: this.outputShutdownStateStore,
      childrenLifetimeContext: this.childrenLifetimeContext,
    });

    this.renderer = new Renderer({
      rootElement,
      onUpdate,
      idPrefix: `${this.outputId}-`,
    });
  }

  public scene(): { video?: Api.Video; audio?: Api.Audio; schedule_time_ms: number } {
    const audio = this.supportsAudio
      ? intoAudioInputsConfiguration(this.audioContext.getAudioConfig())
      : undefined;
    const video = this.supportsVideo ? { root: this.renderer.scene() } : undefined;
    return {
      video,
      audio,
      schedule_time_ms: this.timeContext.timestampMs(),
    };
  }

  public async scheduleAllUpdates(): Promise<void> {
    this.updateTracker = new UpdateTracker();

    while (this.timeContext.timestampMs() <= (this.durationMs ?? Infinity)) {
      while (true) {
        await waitForBlockingTasks(this.timeContext);
        await this.updateTracker.waitForRenderEnd();
        if (!this.timeContext.isBlocked()) {
          break;
        }
      }

      const scene = this.scene();
      await this.api.updateScene(this.outputId, scene);

      const timestampMs = this.timeContext.timestampMs();
      if (this.childrenLifetimeContext.isDone() && this.durationMs === undefined) {
        await this.api.unregisterOutput(OFFLINE_OUTPUT_ID, { schedule_time_ms: timestampMs });
        break;
      }

      this.timeContext.setNextTimestamp();
    }
    this.outputShutdownStateStore.close();
  }
}

class OutputContext implements CompositorOutputContext {
  public readonly globalInputStreamStore: _liveCompositorInternals.InputStreamStore<string>;
  public readonly internalInputStreamStore: _liveCompositorInternals.InputStreamStore<number>;
  public readonly audioContext: _liveCompositorInternals.AudioContext;
  public readonly timeContext: _liveCompositorInternals.TimeContext;
  public readonly outputId: string;
  private output: OfflineOutput;

  constructor(
    output: OfflineOutput,
    outputId: string,
    store: _liveCompositorInternals.InputStreamStore<string>
  ) {
    this.output = output;
    this.globalInputStreamStore = store;
    this.internalInputStreamStore = output.internalInputStreamStore;
    this.audioContext = output.audioContext;
    this.timeContext = output.timeContext;
    this.outputId = outputId;
  }

  public async registerMp4Input(
    inputId: number,
    registerRequest: RegisterMp4Input
  ): Promise<{ videoDurationMs?: number; audioDurationMs?: number }> {
    const inputRef = {
      type: 'output-local',
      outputId: this.outputId,
      id: inputId,
    } as const;
    const offsetMs = this.timeContext.timestampMs();
    const { video_duration_ms: videoDurationMs, audio_duration_ms: audioDurationMs } =
      await this.output.api.registerInput(inputRef, {
        type: 'mp4',
        offset_ms: offsetMs,
        ...registerRequest,
      });
    this.output.internalInputStreamStore.addInput({
      inputId,
      offsetMs,
      videoDurationMs,
      audioDurationMs,
    });
    if (registerRequest.offsetMs) {
      this.timeContext.addTimestamp({ timestamp: offsetMs });
    }
    if (videoDurationMs) {
      this.timeContext.addTimestamp({
        timestamp: (registerRequest.offsetMs ?? 0) + videoDurationMs,
      });
    }
    if (audioDurationMs) {
      this.timeContext.addTimestamp({
        timestamp: (registerRequest.offsetMs ?? 0) + audioDurationMs,
      });
    }
    return {
      videoDurationMs,
      audioDurationMs,
    };
  }
  public async unregisterMp4Input(inputId: number): Promise<void> {
    await this.output.api.unregisterInput(
      {
        type: 'output-local',
        outputId: this.outputId,
        id: inputId,
      },
      { schedule_time_ms: this.timeContext.timestampMs() }
    );
  }
}

async function waitForBlockingTasks(offlineContext: OfflineTimeContext): Promise<void> {
  while (offlineContext.isBlocked()) {
    await sleep(100);
  }
}

const MAX_NO_UPDATE_TIMEOUT_MS = 200;
const MAX_RENDER_TIMEOUT_MS = 2000;

/**
 * Instance that tracks updates, this utils allows us to
 * to monitor when last update happened in the react tree.
 *
 * If there were no updates for MAX_NO_UPDATE_TIMEOUT_MS or
 * MAX_RENDER_TIMEOUT_MS already passed since we started rendering
 * specific PTS then assume it's ready to grab a snapshot of a tree
 */
class UpdateTracker {
  private promise: Promise<void> = new Promise(() => {});
  private promiseRes: () => void = () => {};
  private updateTimeout: number = -1;
  private renderTimeout: number = -1;

  constructor() {
    this.promise = new Promise((res, _rej) => {
      this.promiseRes = res;
    });
    this.onUpdate();
  }

  public onUpdate() {
    clearTimeout(this.updateTimeout);
    this.updateTimeout = setTimeout(() => {
      this.promiseRes();
    }, MAX_NO_UPDATE_TIMEOUT_MS);
  }

  public async waitForRenderEnd(): Promise<void> {
    this.promise = new Promise((res, _rej) => {
      this.promiseRes = res;
    });
    clearTimeout(this.renderTimeout);
    this.renderTimeout = setTimeout(() => {
      console.warn(
        "Render for a specific timestamp took too long, make sure you don't have infinite update loop."
      );
      this.promiseRes();
    }, MAX_RENDER_TIMEOUT_MS);
    await this.promise;
    clearTimeout(this.renderTimeout);
    clearTimeout(this.updateTimeout);
  }
}

export default OfflineOutput;
