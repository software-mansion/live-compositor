import type { Component, ImageSpec, Renderer } from '@live-compositor/browser-render';
import type { Framerate } from '../compositor/compositor';
import type { Logger } from 'pino';
import type { Api } from 'live-compositor';
import { createInput } from './input/input';
import { Output } from './output/output';
import { Queue } from './queue';
import type { RegisterInput, RegisterOutput, WorkerEvent, WorkerResponse } from '../workerApi';
import { workerPostEvent as genericWorkerPostEvent } from './bridge';
import { CompositorEventType } from '../eventSender';

export const workerPostEvent = genericWorkerPostEvent<WorkerEvent>;

export class Pipeline {
  private renderer: Renderer;
  private queue: Queue;
  private logger: Logger;
  private started = false;

  public constructor(options: { renderer: Renderer; framerate: Framerate; logger: Logger }) {
    this.renderer = options.renderer;
    this.logger = options.logger.child({ element: 'pipeline' });
    this.queue = new Queue(options.framerate, options.renderer, options.logger);
  }

  public start() {
    if (this.started) {
      throw new Error('Compositor was already started');
    }
    this.started = true;
    this.queue.start();
  }

  public async terminate(): Promise<void> {
    this.queue.stop();
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<WorkerResponse> {
    const input = await createInput(inputId, request, this.logger);
    // `addInput` will throw an exception if input already exists
    this.queue.addInput(inputId, input);
    await this.renderer.registerInput(inputId);
    const result = input.start();
    return {
      type: 'registerInput',
      body: {
        video_duration_ms: result.videoDurationMs,
        audio_duration_ms: result.audioDurationMs,
      },
    };
  }

  public async unregisterInput(inputId: string): Promise<void> {
    this.queue.removeInput(inputId);
    await this.renderer.unregisterInput(inputId);
  }

  public async registerOutput(outputId: string, request: RegisterOutput) {
    if (request.video) {
      const output = new Output(request);
      this.queue.addOutput(outputId, output);
      try {
        // `updateScene` implicitly registers the output.
        // In case of an error, the output has to be manually cleaned up from the renderer.
        await this.renderer.updateScene(
          outputId,
          request.video.resolution,
          request.video.initial.root as Component
        );
      } catch (e) {
        this.queue.removeOutput(outputId);
        await this.renderer.unregisterOutput(outputId);
        throw e;
      }
    }
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    this.queue.removeOutput(outputId);
    await this.renderer.unregisterOutput(outputId);
    // If we add outputs that can end early or require flushing
    // then this needs to be change
    workerPostEvent({
      type: CompositorEventType.OUTPUT_DONE,
      outputId,
    });
  }

  public async updateScene(outputId: string, request: Api.UpdateOutputRequest) {
    if (!request.video) {
      return;
    }
    const output = this.queue.getOutput(outputId);
    if (!output) {
      throw new Error(`Unknown output "${outputId}"`);
    }
    await this.renderer.updateScene(outputId, output.resolution, request.video.root as Component);
  }

  public async registerImage(imageId: string, request: ImageSpec) {
    await this.renderer.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string) {
    await this.renderer.unregisterImage(imageId);
  }

  public async registerFont(url: string): Promise<void> {
    await this.renderer.registerFont(url);
  }
}
