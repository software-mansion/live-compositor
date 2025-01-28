import type { CompositorManager, RegisterInput, RegisterOutput } from '@live-compositor/core';
import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import { createLogger } from '../logger';
import LocallySpawnedInstance from '../manager/locallySpawnedInstance';
import assert from 'assert';
import type { ReactElement } from 'react';
import type { RegisterImage } from '../../../../live-compositor/cjs/types/registerRenderer';
import FormData from 'form-data';
import fetch from 'node-fetch';

export default class LiveCompositor {
  private coreCompositor?: CoreLiveCompositor;

  public constructor(manager?: CompositorManager) {
    this.coreCompositor = new CoreLiveCompositor(
      manager ?? LocallySpawnedInstance.defaultManager(),
      createLogger()
    );
  }

  public async init(): Promise<void> {
    await this.coreCompositor!.init();
  }

  public async registerOutput(
    outputId: string,
    root: ReactElement,
    request: RegisterOutput
  ): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerOutput(outputId, root, request);
  }

  public async unregisterOutput(outputId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterOutput(outputId);
  }

  public async registerInput(inputId: string, request: RegisterInput): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerInput(inputId, request);
  }

  public async unregisterInput(inputId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterInput(inputId);
  }

  public async registerImage(imageId: string, request: RegisterImage): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.registerImage(imageId, request);
  }

  public async unregisterImage(imageId: string): Promise<void> {
    assert(this.coreCompositor);
    await this.coreCompositor.unregisterImage(imageId);
  }

  public async registerFont(fontSource: string | ArrayBuffer): Promise<object> {
    let fontBuffer: Buffer;

    if (fontSource instanceof ArrayBuffer) {
      fontBuffer = Buffer.from(fontSource);
    } else {
      const response = await fetch(fontSource);
      if (!response.ok) {
        throw new Error(`Failed to fetch the font file from ${fontSource}`);
      }
      fontBuffer = await response.buffer();
    }

    const formData = new FormData();
    formData.append('fontFile', fontBuffer);

    assert(this.coreCompositor);
    return this.coreCompositor.manager.sendMultipartRequest({
      method: 'POST',
      route: `/api/font/register`,
      body: formData,
    });
  }

  public async start(): Promise<void> {
    await this.coreCompositor!.start();
  }

  public async terminate(): Promise<void> {
    await this.coreCompositor?.terminate();
  }
}
