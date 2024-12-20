import type { CompositorManager } from '@live-compositor/core';
import {
  LiveCompositor as CoreLiveCompositor,
  OfflineCompositor as CoreOfflineCompositor,
} from '@live-compositor/core';
import LocallySpawnedInstance from './manager/locallySpawnedInstance';
import ExistingInstance from './manager/existingInstance';
import { createLogger } from './logger';

export { LocallySpawnedInstance, ExistingInstance };

export default class LiveCompositor extends CoreLiveCompositor {
  constructor(manager?: CompositorManager) {
    super(manager ?? LocallySpawnedInstance.defaultManager(), createLogger());
  }
}

export class OfflineCompositor extends CoreOfflineCompositor {
  constructor(manager?: CompositorManager) {
    super(manager ?? LocallySpawnedInstance.defaultManager(), createLogger());
  }
}
