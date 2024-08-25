import { ContextStore, LiveCompositorContext, RegisterOutput } from 'live-compositor';
import { ApiClient, Api } from './api';
import Renderer from './renderer';
import { intoAudioInputsConfiguration } from './api/output';
import React from 'react';

class Output {
  api: ApiClient;
  outputId: string;
  initialized: boolean = false;

  throttledUpdate: () => void;
  videoRenderer?: Renderer;

  constructor(outputId: string, output: RegisterOutput, api: ApiClient, store: ContextStore) {
    this.api = api;
    this.outputId = outputId;

    let audioOptions: Api.Audio | undefined;
    if (output.video) {
      this.videoRenderer = new Renderer(
        React.createElement(LiveCompositorContext.Provider, { value: store }, output.video.root),
        () => this.onRendererUpdate(),
        `${outputId}-`
      );
    }
    if (output.audio) {
      audioOptions = intoAudioInputsConfiguration(output.audio.initial);
    }
    this.throttledUpdate = throttle(async () => {
      await api.updateScene(this.outputId, {
        video: this.videoRenderer && { root: this.videoRenderer.scene() },
        audio: audioOptions,
      });
    }, 30);
  }

  public scene(): Api.Video | undefined {
    return this.videoRenderer && { root: this.videoRenderer.scene() };
  }

  private onRendererUpdate() {
    if (!this.throttledUpdate || !this.videoRenderer) {
      return;
    }
    this.throttledUpdate();
  }
}

function throttle(fn: () => Promise<void>, timeoutMs: number): () => void {
  let shouldCall: boolean = false;
  let running: boolean = false;

  const start = async () => {
    while (shouldCall) {
      const start = Date.now();
      shouldCall = false;

      try {
        await fn();
      } catch (error) {
        console.log(error);
      }

      const timeoutLeft = start + timeoutMs - Date.now();
      if (timeoutLeft > 0) {
        await sleep(timeoutLeft);
      }
      running = false;
    }
  };

  return () => {
    shouldCall = true;
    if (running) {
      return;
    }
    running = true;
    void start();
  };
}

async function sleep(timeout_ms: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeout_ms);
  });
}

export default Output;
