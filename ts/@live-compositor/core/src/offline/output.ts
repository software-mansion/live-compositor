import type { Outputs } from 'live-compositor';
import { _liveCompositorInternals, View } from 'live-compositor';
import type React from 'react';
import { createElement, useSyncExternalStore } from 'react';
import type { ApiClient, Api } from '../api.js';
import Renderer from '../renderer.js';
import type { RegisterOutput } from '../api/output.js';
import { intoAudioInputsConfiguration } from '../api/output.js';
import { sleep } from '../utils.js';

type AudioContext = _liveCompositorInternals.AudioContext;
type OfflineTimeContext = _liveCompositorInternals.OfflineTimeContext;
type OfflineInstanceContextStore = _liveCompositorInternals.OfflineInstanceContextStore;

class OfflineOutput {
  api: ApiClient;
  outputId: string;
  audioContext: AudioContext;
  timeContext: OfflineTimeContext;
  outputShutdownStateStore: OutputShutdownStateStore;
  durationMs: number;
  updateTracker?: UpdateTracker;

  videoRenderer?: Renderer;
  initialAudioConfig?: Outputs.AudioInputsConfiguration;

  constructor(
    outputId: string,
    registerRequest: RegisterOutput,
    api: ApiClient,
    store: OfflineInstanceContextStore,
    durationMs: number
  ) {
    this.api = api;
    this.outputId = outputId;
    this.outputShutdownStateStore = new OutputShutdownStateStore();
    this.durationMs = durationMs;

    const supportsAudio = 'audio' in registerRequest && !!registerRequest.audio;
    if (supportsAudio) {
      this.initialAudioConfig = registerRequest.audio!.initial ?? { inputs: [] };
    }

    const onUpdate = () => this.updateTracker?.onUpdate();
    this.audioContext = new _liveCompositorInternals.AudioContext(onUpdate, supportsAudio);
    this.timeContext = new _liveCompositorInternals.OfflineTimeContext(onUpdate, store);

    if (registerRequest.video) {
      const rootElement = createElement(OutputRootComponent, {
        instanceStore: store,
        audioContext: this.audioContext,
        timeContext: this.timeContext,
        outputRoot: registerRequest.video.root,
        outputShutdownStateStore: this.outputShutdownStateStore,
      });

      this.videoRenderer = new Renderer({
        rootElement,
        onUpdate,
        idPrefix: `${outputId}-`,
      });
    }
  }

  public scene(): { video?: Api.Video; audio?: Api.Audio; schedule_time_ms: number } {
    const audio = this.audioContext.getAudioConfig() ?? this.initialAudioConfig;
    return {
      video: this.videoRenderer && { root: this.videoRenderer.scene() },
      audio: audio && intoAudioInputsConfiguration(audio),
      schedule_time_ms: this.timeContext.timestampMs(),
    };
  }

  public async scheduleAllUpdates(): Promise<void> {
    this.updateTracker = new UpdateTracker();
    while (this.timeContext.timestampMs() <= this.durationMs) {
      console.log('Event loop', this.timeContext.timestampMs());
      while (true) {
        await waitForBlockingTasks(this.timeContext);
        await this.updateTracker.waitForRenderEnd();
        if (!this.timeContext.isBlocked()) {
          break;
        }
      }

      const scene = this.scene();
      await this.api.updateScene(this.outputId, scene);
      this.timeContext.setNextTimestamp();
    }

    this.outputShutdownStateStore.close();
  }
}

async function waitForBlockingTasks(offlineContext: OfflineTimeContext): Promise<void> {
  while (offlineContext.isBlocked()) {
    await sleep(100);
  }
}

const MAX_NO_UPDATE_TIMEOUT_MS = 20;
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
  private promise: Promise<void> = new Promise(() => { });
  private promiseRes: () => void = () => { };
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

// External store to share shutdown information between React tree
// and external code that is managing it.
class OutputShutdownStateStore {
  private shutdown: boolean = false;
  private onChangeCallbacks: Set<() => void> = new Set();

  public close() {
    this.shutdown = true;
    this.onChangeCallbacks.forEach(cb => cb());
  }

  // callback for useSyncExternalStore
  public getSnapshot = (): boolean => {
    return this.shutdown;
  };

  // callback for useSyncExternalStore
  public subscribe = (onStoreChange: () => void): (() => void) => {
    this.onChangeCallbacks.add(onStoreChange);
    return () => {
      this.onChangeCallbacks.delete(onStoreChange);
    };
  };
}

function OutputRootComponent({
  outputRoot,
  instanceStore,
  timeContext,
  audioContext,
  outputShutdownStateStore,
}: {
  outputRoot: React.ReactElement;
  instanceStore: InstanceContextStore;
  timeContext: OfflineTimeContext;
  audioContext: AudioContext;
  outputShutdownStateStore: OutputShutdownStateStore;
}) {
  const shouldShutdown = useSyncExternalStore(
    outputShutdownStateStore.subscribe,
    outputShutdownStateStore.getSnapshot
  );

  if (shouldShutdown) {
    // replace root with view to stop all the dynamic code
    return createElement(View, {});
  }

  const reactCtx = {
    instanceStore,
    timeContext,
    audioContext,
  };
  return createElement(
    _liveCompositorInternals.LiveCompositorContext.Provider,
    { value: reactCtx },
    outputRoot
  );
}

export default OfflineOutput;
