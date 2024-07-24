import { RegisterOutput, SceneComponent } from 'live-compositor';
import { ApiClient, Api } from './api';
import Renderer from './renderer';
import { intoAudioInputsConfiguration } from './api/output';

class Output {
  api: ApiClient;
  outputId: string;
  initialized: boolean = false;

  videoRenderer?: Renderer;
  audioOptions?: Api.Audio;

  constructor(outputId: string, output: RegisterOutput, api: ApiClient) {
    this.api = api;
    this.outputId = outputId;

    if (output.video) {
      this.videoRenderer = new Renderer(
        output.video.root,
        scene => this.onRendererUpdate(scene),
        `${outputId}-`
      );
    }
    if (output.audio) {
      this.audioOptions = intoAudioInputsConfiguration(output.audio.initial);
    }
    this.initialized = true;
  }

  public scene(): { video?: Api.Video; audio?: Api.Audio } {
    return {
      video: this.videoRenderer && sceneToVideo(this.videoRenderer?.scene()),
      audio: this.audioOptions,
    };
  }

  private onRendererUpdate(scene: SceneComponent[]) {
    if (!this.initialized) {
      return;
    }
    // TODO: make sure that only one request is sent at the time
    this.api.updateScene(this.outputId, {
      video: sceneToVideo(scene),
      audio: this.audioOptions,
    });
  }
}

function sceneToVideo(scene: SceneComponent[]): Api.Video {
  if (scene.length !== 1 || typeof scene[0] === 'string') {
    throw new Error('Component should return exactly one component.');
  }
  return { root: scene[0] };
}

export default Output;
