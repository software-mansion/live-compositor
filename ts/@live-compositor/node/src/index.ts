import {
  LiveCompositor as CoreLiveCompositor,
  CompositorManager,
  createLiveCompositor,
} from '@live-compositor/core';
import LocallySpawnedInstance from './manager/locallySpawnedInstance';
import ExistingInstance from './manager/existingInstance';

export { LocallySpawnedInstance, ExistingInstance };

export default class LiveCompositor extends CoreLiveCompositor {
  public static async create(manager?: CompositorManager): Promise<LiveCompositor> {
    return await createLiveCompositor(manager ?? LocallySpawnedInstance.defaultManager());
  }
}
