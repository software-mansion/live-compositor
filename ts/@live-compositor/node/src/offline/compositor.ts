import type {
  Input as CoreInput,
  Output as CoreOutput,
  CompositorManager,
} from '@live-compositor/core';
import { OfflineCompositor as CoreLiveCompositor } from '@live-compositor/core';
import { createLogger } from '../logger';
import LocallySpawnedInstance from '../manager/locallySpawnedInstance';
import type { ReactElement } from 'react';
import type { Renderers } from 'live-compositor';
import fetch from 'node-fetch';
import FormData from 'form-data';

export default class OfflineCompositor {
  private coreCompositor: CoreLiveCompositor;

  public constructor(manager?: CompositorManager) {
    this.coreCompositor = new CoreLiveCompositor(
      manager ?? LocallySpawnedInstance.defaultManager(),
      createLogger()
    );
  }

  public async init(): Promise<void> {
    await this.coreCompositor.init();
  }

  public async render(root: ReactElement, request: CoreOutput.RegisterOutput, durationMs?: number) {
    await this.coreCompositor.render(root, request, durationMs);
  }

  public async registerInput(inputId: string, request: CoreInput.RegisterInput) {
    await this.coreCompositor.registerInput(inputId, request);
  }

  public async registerShader(shaderId: string, request: Renderers.RegisterShader) {
    await this.coreCompositor.registerShader(shaderId, request);
  }

  public async registerImage(imageId: string, request: Renderers.RegisterImage) {
    await this.registerImage(imageId, request);
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

    return this.coreCompositor.manager.sendMultipartRequest({
      method: 'POST',
      route: `/api/font/register`,
      body: formData,
    });
  }
}
