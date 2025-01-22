import { CompositorManager, RegisterInput, RegisterOutput } from '@live-compositor/core';
import { OfflineCompositor as CoreLiveCompositor } from '@live-compositor/core';
import { createLogger } from '../logger';
import LocallySpawnedInstance from '../manager/locallySpawnedInstance';
import assert from 'assert';
import type { ReactElement } from 'react';
import { Renderers } from 'live-compositor';

export default class OfflineCompositor {
  private coreCompositor?: CoreLiveCompositor;

  public constructor(manager?: CompositorManager) {
    this.coreCompositor = new CoreLiveCompositor(manager ?? LocallySpawnedInstance.defaultManager(), createLogger())
  }

  public async init(): Promise<void> {
    await this.coreCompositor!.init();
  }

  public async render(root: ReactElement, request: RegisterOutput, durationMs?: number) {
    assert(this.coreCompositor)
    await this.coreCompositor.render(root, request, durationMs);
  }

  public async registerInput(inputId: string, request: RegisterInput) {
    assert(this.coreCompositor)
    await this.coreCompositor.registerInput(inputId, request);
  }

  public async registerShader(
    shaderId: string,
    request: Renderers.RegisterShader
  ){
    assert(this.coreCompositor);
    await this.coreCompositor.registerShader(shaderId, request);
  }

  public async registerImage(imageId: string, request: Renderers.RegisterImage) {
    assert(this.coreCompositor)
    await this.registerImage(imageId, request);

  }
}
