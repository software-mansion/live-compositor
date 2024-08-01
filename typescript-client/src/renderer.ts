import Reconciler from 'react-reconciler';

import {
  batchedUpdates as batchedUpdatesImpl,
  createContainer,
  updateContainer,
  getPublicRootInstance,
  defaultOnRecoverableError,
} from 'react-reconciler/src/ReactFiberReconciler';

const HostConfig: Reconciler.HostConfig<> = {
  supportsPersistence: true,
  // You'll need to implement some methods here.
  // See below for more information and examples.
};

const MyRenderer = Reconciler(HostConfig);

const RendererPublicAPI = {
  render(element, container, callback) {
    const root = createContainer(
      container,
      LegacyRoot,
      null,
      false,
      null,
      '',
      onUncaughtError,
      onCaughtError,
      onRecoverableError,
      null
    );

    updateContainer(element, root, null, callback);

    return getPublicRootInstance(root);
  },
};

module.exports = RendererPublicAPI;
