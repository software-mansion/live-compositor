import type { CompositorManager } from '@live-compositor/core';
import { LiveCompositor as CoreLiveCompositor } from '@live-compositor/core';
import LocallySpawnedInstance from './manager/locallySpawnedInstance';
import ExistingInstance from './manager/existingInstance';

export { LocallySpawnedInstance, ExistingInstance };

export default class LiveCompositor extends CoreLiveCompositor {
  constructor(manager?: CompositorManager) {
    super(manager ?? LocallySpawnedInstance.defaultManager());
  }
}
