import type { RegisterMp4Input } from 'live-compositor';
import { _liveCompositorInternals } from 'live-compositor';
import type { ReactElement } from 'react';
import { createElement } from 'react';
import type { ApiClient, Api } from '../api.js';
import Renderer from '../renderer.js';
import type { RegisterOutput } from '../api/output.js';
import { intoAudioInputsConfiguration } from '../api/output.js';
import { ThrottledFunction } from '../utils.js';
import { OutputRootComponent } from '../rootComponent.js';
import type { Logger } from 'pino';

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
  logger: Logger;

  shouldUpdateWhenReady: boolean = false;
  throttledUpdate: ThrottledFunction;

  supportsAudio: boolean;
  supportsVideo: boolean;

  renderer: Renderer;

  constructor(
    outputId: string,
    root: ReactElement,
    registerRequest: RegisterOutput,
    api: ApiClient,
    store: LiveInputStreamStore<string>,
    startTimestamp: number | undefined,
    logger: Logger
  ) {
    this.api = api;
    this.logger = logger;
    this.outputId = outputId;
    this.shouldUpdateWhenReady = false;
    this.throttledUpdate = new ThrottledFunction(
      async () => {
        this.shouldUpdateWhenReady = true;
      },
      {
        timeoutMs: 30,
        logger: this.logger,
      }
    );

    this.supportsAudio = 'audio' in registerRequest && !!registerRequest.audio;
    this.supportsVideo = 'video' in registerRequest && !!registerRequest.video;

    const onUpdate = () => this.throttledUpdate.scheduleCall();
    this.audioContext = new _liveCompositorInternals.AudioContext(onUpdate);
    this.timeContext = new _liveCompositorInternals.LiveTimeContext();
    this.internalInputStreamStore = new _liveCompositorInternals.LiveInputStreamStore(this.logger);
    if (startTimestamp !== undefined) {
      this.timeContext.initClock(startTimestamp);
    }

    const rootElement = createElement(OutputRootComponent, {
      outputContext: new OutputContext(this, this.outputId, store),
      outputRoot: root,
      childrenLifetimeContext: new _liveCompositorInternals.ChildrenLifetimeContext(() => {}),
    });

    this.renderer = new Renderer({
      rootElement,
      onUpdate,
      idPrefix: `${outputId}-`,
      logger: logger.child({ element: 'react-renderer' }),
    });
  }

  public scene(): { video?: Api.Video; audio?: Api.Audio } {
    const audio = this.supportsAudio
      ? intoAudioInputsConfiguration(this.audioContext.getAudioConfig())
      : undefined;
    const video = this.supportsVideo ? { root: this.renderer.scene() } : undefined;
    return {
      audio,
      video,
    };
  }

  public async close(): Promise<void> {
    this.throttledUpdate.setFn(async () => {});
    this.renderer.stop();
    await this.throttledUpdate.waitForPendingCalls();
  }

  public async ready() {
    this.throttledUpdate.setFn(async () => {
      await this.api.updateScene(this.outputId, this.scene());
    });
    if (this.shouldUpdateWhenReady) {
      this.throttledUpdate.scheduleCall();
    }
  }

  public initClock(timestamp: number) {
    this.timeContext.initClock(timestamp);
  }

  public inputStreamStore(): LiveInputStreamStore<number> {
    return this.internalInputStreamStore;
  }
}

class OutputContext implements CompositorOutputContext {
  public readonly globalInputStreamStore: _liveCompositorInternals.InputStreamStore<string>;
  public readonly internalInputStreamStore: _liveCompositorInternals.InputStreamStore<number>;
  public readonly audioContext: _liveCompositorInternals.AudioContext;
  public readonly timeContext: _liveCompositorInternals.TimeContext;
  public readonly outputId: string;
  public readonly logger: Logger;
  private output: Output;

  constructor(
    output: Output,
    outputId: string,
    store: _liveCompositorInternals.InputStreamStore<string>
  ) {
    this.output = output;
    this.globalInputStreamStore = store;
    this.internalInputStreamStore = output.internalInputStreamStore;
    this.audioContext = output.audioContext;
    this.timeContext = output.timeContext;
    this.outputId = outputId;
    this.logger = output.logger;
  }

  public async registerMp4Input(
    inputId: number,
    registerRequest: RegisterMp4Input
  ): Promise<{ videoDurationMs?: number; audioDurationMs?: number }> {
    return await this.output.internalInputStreamStore.runBlocking(async updateStore => {
      const inputRef = {
        type: 'output-local',
        outputId: this.outputId,
        id: inputId,
      } as const;
      const { video_duration_ms: videoDurationMs, audio_duration_ms: audioDurationMs } =
        await this.output.api.registerInput(inputRef, {
          type: 'mp4',
          ...registerRequest,
        });
      updateStore({
        type: 'add_input',
        input: {
          inputId: inputId,
          offsetMs: registerRequest.offsetMs,
          audioDurationMs,
          videoDurationMs,
        },
      });
      return {
        audioDurationMs,
        videoDurationMs,
      };
    });
  }

  public async unregisterMp4Input(inputId: number): Promise<void> {
    await this.output.api.unregisterInput(
      {
        type: 'output-local',
        outputId: this.outputId,
        id: inputId,
      },
      {}
    );
  }
}

export default Output;
