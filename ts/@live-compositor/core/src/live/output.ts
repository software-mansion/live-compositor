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
type LiveInputStreamStore<Id> = _liveCompositorInternals.LiveInputStreamStore<Id>;
type CompositorOutputContext = _liveCompositorInternals.CompositorOutputContext;

class Output {
  api: ApiClient;
  outputId: string;
  audioContext: AudioContext;
  timeContext: LiveTimeContext;
  internalInputStreamStore: LiveInputStreamStore<number>;
  outputShutdownStateStore: OutputShutdownStateStore;

  shouldUpdateWhenReady: boolean = false;
  throttledUpdate: () => void;
  videoRenderer?: Renderer;
  initialAudioConfig?: Outputs.AudioInputsConfiguration;

  constructor(
    outputId: string,
    registerRequest: RegisterOutput,
    api: ApiClient,
    store: LiveInputStreamStore<string>,
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
    this.internalInputStreamStore = new _liveCompositorInternals.LiveInputStreamStore();
    if (startTimestamp !== undefined) {
      this.timeContext.initClock(startTimestamp);
    }

    if (registerRequest.video) {
      const rootElement = createElement(OutputRootComponent, {
        outputContext: {
          globalInputStreamStore: store,
          internalInputStreamStore: this.internalInputStreamStore,
          audioContext: this.audioContext,
          timeContext: this.timeContext,
          outputId,
          registerMp4Input: async (inputId, registerRequest) => {
            // TODO: refactor outside of a constructor
            return await this.internalInputStreamStore.runBlocking(async updateStore => {
              const inputRef = {
                type: 'output-local',
                outputId,
                id: inputId,
              } as const;
              const { video_duration_ms, audio_duration_ms } = await this.api.registerInput(
                inputRef,
                {
                  type: 'mp4',
                  ...registerRequest,
                }
              );
              updateStore({
                type: 'add_input',
                input: {
                  inputId: inputId,
                  offsetMs: registerRequest.offsetMs,
                  audioDurationMs: audio_duration_ms,
                  videoDurationMs: video_duration_ms,
                },
              });
              return {
                audioDurationMs: audio_duration_ms,
                videoDurationMs: video_duration_ms,
              };
            });
          },
        },
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

  public inputStreamStore(): LiveInputStreamStore<number> {
    return this.internalInputStreamStore;
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
  outputContext,
  outputRoot,
  outputShutdownStateStore,
}: {
  outputContext: CompositorOutputContext;
  outputRoot: React.ReactElement;
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

  return createElement(
    _liveCompositorInternals.LiveCompositorContext.Provider,
    { value: outputContext },
    outputRoot
  );
}

export default Output;
