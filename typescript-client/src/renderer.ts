import Reconciler from 'react-reconciler';

import {
  createContainer,
  updateContainer,
  getPublicRootInstance,
} from 'react-reconciler/src/ReactFiberReconciler';

class LiveCompositorHostComponent {}
type HostContext = {}

const HostConfig: Reconciler.HostConfig<
  string,
  object,
  number,
  LiveCompositorHostComponent,
  number, //TextInstance,
  void, // SuspenseInstance,
  void, // HydratableInstance,
  LiveCompositorHostComponent, //PublicInstance,
  HostContext, //HostContext,
  object, // UpdatePayload,
  void, // ChildSet,
  NodeJS.Timeout, // TimeoutHandle,
  -1 // NoTimeout
> = {
  supportsPersistence: true,

  createInstance(
    type,
    props,
    rootContainer,
    hostContext,
    internalHandle
  ): LiveCompositorHostComponent {
    console.log('this.createInstance', type, props, rootContainer, hostContext, internalHandle);
    return new LiveCompositorHostComponent();
  },
};

const MyRenderer = Reconciler(HostConfig);

const RendererPublicAPI = {
  render(element, container, callback) {
    const root = MyRenderer.createContainer(
      container,
      new LiveCompositorHostComponent(),
      null,
      false,
      null,
      '',
      console.error, //onUncaughtError,
      console.error, // onCaughtError,
      console.error, //onRecoverableError,
      null
    );

    MyRenderer.updateContainer(element, root, null, callback);

    return getPublicRootInstance(root);
  },
};

module.exports = RendererPublicAPI;
