import Api, * as ApiTypes from './api';
import React from 'react';
import Renderer from './renderer';
import { SceneComponent } from './component';

export type RegisterOutput = Omit<ApiTypes.RegisterOutput, 'video'> & {
  video?: Omit<ApiTypes.OutputRtpVideoOptions, 'initial'> & {
    root: React.ReactElement;
  };
};

class OutputRenderer {
  api: Api;
  outputId: string;
  initialized: boolean = false;

  videoRenderer?: Renderer;
  audioOptions?: ApiTypes.Audio;

  constructor(outputId: string, output: RegisterOutput, api: Api) {
    this.api = api;
    this.outputId = outputId;

    if (output.video) {
      this.videoRenderer = new Renderer(output.video.root, scene => this.onRendererUpdate(scene));
    }
    if (output.audio) {
      this.audioOptions = output.audio.initial;
    }
    this.initialized = true;
  }

  public scene(): { video?: ApiTypes.Video; audio?: ApiTypes.Audio } {
    return {
      video: this.videoRenderer && sceneToVideo(this.videoRenderer?.scene()),
      audio: this.audioOptions,
    };
  }

  private onRendererUpdate(scene: SceneComponent[]) {
    if (!this.initialized) {
      return;
    }
    console.log('update', scene);
    // TODO: make sure that only one request is sent at the time
    this.api.updateScene(this.outputId, {
      video: sceneToVideo(scene),
      audio: this.audioOptions,
    });
  }
}

function sceneToVideo(scene: SceneComponent[]): ApiTypes.Video {
  if (scene.length !== 1 || typeof scene[0] === 'string') {
    throw new Error('Component should return exactly one component.');
  }
  return { root: scene[0] };
}

export default OutputRenderer;
