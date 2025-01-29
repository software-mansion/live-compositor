import type { RegisterInput, RegisterOutput, NodeCompositorManager } from '@live-compositor/core';
import { OfflineCompositor as CoreLiveCompositor } from '@live-compositor/core';
import { createLogger } from '../logger';
import LocallySpawnedInstance from '../manager/locallySpawnedInstance';
import assert from 'assert';
import type { ReactElement } from 'react';
import type { Renderers } from 'live-compositor';
import fetch from 'node-fetch';
import FormData from 'form-data';

export default class OfflineCompositor {
  private coreCompositor?: CoreLiveCompositor;
  private nodeCompositorManager: NodeCompositorManager;

  public constructor(manager?: NodeCompositorManager) {
    this.nodeCompositorManager = manager ?? LocallySpawnedInstance.defaultManager();
    this.coreCompositor = new CoreLiveCompositor(this.nodeCompositorManager, createLogger());
  }

  public async init(): Promise<void> {
    await this.coreCompositor!.init();
  }

  public async render(root: ReactElement, request: RegisterOutput, durationMs?: number) {
    assert(this.coreCompositor);
    await this.coreCompositor.render(root, request, durationMs);
  }

  public async registerInput(inputId: string, request: RegisterInput) {
    assert(this.coreCompositor);
    await this.coreCompositor.registerInput(inputId, request);
  }

  public async registerShader(shaderId: string, request: Renderers.RegisterShader) {
    assert(this.coreCompositor);
    await this.coreCompositor.registerShader(shaderId, request);
  }

  public async registerImage(imageId: string, request: Renderers.RegisterImage) {
    assert(this.coreCompositor);
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

    return this.nodeCompositorManager.sendMultipartRequest({
      method: 'POST',
      route: `/api/font/register`,
      body: formData,
    });
  }
}
