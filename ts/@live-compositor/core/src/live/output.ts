import type { Outputs } from 'live-compositor';
import { _liveCompositorInternals, View } from 'live-compositor';
import type React from 'react';
import { createElement, useSyncExternalStore } from 'react';
import type { ApiClient, Api } from '../api.js';
import Renderer from '../renderer.js';
import type { RegisterOutput } from '../api/output.js';
import { intoAudioInputsConfiguration } from '../api/output.js';
import { throttle } from '../utils.js';

type AudioContext = _liveCompositorInternals.AudioContext;
type LiveTimeContext = _liveCompositorInternals.LiveTimeContext;
type InstanceContextStore = _liveCompositorInternals.InstanceContextStore;

class Output {
  api: ApiClient;
  outputId: string;
  audioContext: AudioContext;
  timeContext: LiveTimeContext;
  outputShutdownStateStore: OutputShutdownStateStore;

  shouldUpdateWhenReady: boolean = false;
  throttledUpdate: () => void;
  videoRenderer?: Renderer;
  initialAudioConfig?: Outputs.AudioInputsConfiguration;

  constructor(
    outputId: string,
    registerRequest: RegisterOutput,
    api: ApiClient,
    store: InstanceContextStore,
    startTimestamp: number | undefined
  ) {
    this.api = api;
    this.outputId = outputId;
    this.outputShutdownStateStore = new OutputShutdownStateStore();
    this.shouldUpdateWhenReady = false;
    this.throttledUpdate = () => {
      this.shouldUpdateWhenReady = true;
    };

    const supportsAudio = 'audio' in registerRequest && !!registerRequest.audio;
    if (supportsAudio) {
      this.initialAudioConfig = registerRequest.audio!.initial ?? { inputs: [] };
    }

    const onUpdate = () => this.throttledUpdate();
    this.audioContext = new _liveCompositorInternals.AudioContext(onUpdate, supportsAudio);
    this.timeContext = new _liveCompositorInternals.LiveTimeContext();
    if (startTimestamp !== undefined) {
      this.timeContext.initClock(startTimestamp);
    }

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

  public scene(): { video?: Api.Video; audio?: Api.Audio } {
    const audio = this.audioContext.getAudioConfig() ?? this.initialAudioConfig;
    return {
      video: this.videoRenderer && { root: this.videoRenderer.scene() },
      audio: audio && intoAudioInputsConfiguration(audio),
    };
  }

  public close(): void {
    this.throttledUpdate = () => {};
    // close will switch a scene to just a <View />, so we need replace `throttledUpdate`
    // callback before it is called
    this.outputShutdownStateStore.close();
  }

  public async ready() {
    this.throttledUpdate = throttle(async () => {
      await this.api.updateScene(this.outputId, this.scene());
    }, 30);
    if (this.shouldUpdateWhenReady) {
      this.throttledUpdate();
    }
  }

  public initClock(timestamp: number) {
    this.timeContext.initClock(timestamp);
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
  audioContext: AudioContext;
  timeContext: LiveTimeContext;
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

export default Output;
