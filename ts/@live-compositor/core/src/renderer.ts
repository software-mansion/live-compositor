import Reconciler from 'react-reconciler';
import { DefaultEventPriority, LegacyRoot } from 'react-reconciler/constants';
import { Api } from './api';
import { SceneBuilder, SceneComponent } from 'live-compositor';

export class LiveCompositorHostComponent {
  public props: object;
  public sceneBuilder: SceneBuilder<object>;
  public children: (Instance | TextInstance)[] = [];

  constructor(props: object, sceneBuilder: SceneBuilder<object>) {
    this.props = props;
    this.sceneBuilder = sceneBuilder;
  }

  public scene(): Api.Component {
    const children = this.children.map(child =>
      typeof child === 'string' ? child : child.scene()
    );
    return this.sceneBuilder(this.props, groupTextComponents(children));
  }
}

type Type = string;
type Props = {
  props: object;
  sceneBuilder: SceneBuilder<object>;
};
type Container = Renderer;
type HostContext = object;
type Instance = LiveCompositorHostComponent;
type TextInstance = string;
type ChildSet = Array<string | Instance>;

const HostConfig: Reconciler.HostConfig<
  Type,
  Props,
  Container,
  Instance,
  TextInstance,
  void, // SuspenseInstance
  void, // HydratableInstance
  Instance, //PublicInstance
  HostContext,
  object, // UpdatePayload
  ChildSet,
  number, // TimeoutHandle
  -1 // NoTimeout
> = {
  getPublicInstance(instance: Instance | TextInstance) {
    return instance as Instance;
  },

  getRootHostContext(_rootContainer: Container) {
    return null;
  },

  getChildHostContext(
    parentHostContext: HostContext,
    _type: Type,
    _rootContainer: Container
  ): HostContext {
    return parentHostContext;
  },

  prepareForCommit(_containerInfo: Container): Record<string, any> | null {
    return null;
  },

  resetAfterCommit(container: Container): void {
    container.onUpdate();
  },

  createInstance(
    type: Type,
    props: Props,
    _rootContainer: Container,
    _hostContext: HostContext,
    _internalHandle: any
  ): LiveCompositorHostComponent {
    if (type === 'compositor') {
      return new LiveCompositorHostComponent(props.props, props.sceneBuilder);
    } else {
      throw new Error(`Unknown type ${type}`);
    }
  },

  appendInitialChild(parentInstance: Instance, child: Instance | TextInstance): void {
    parentInstance.children.push(child);
  },

  finalizeInitialChildren(
    _instance: Instance,
    _type: Type,
    _props: Props,
    _rootContainer: Container,
    _hostContext: HostContext
  ): boolean {
    // if true commitMount will be called
    return false;
  },

  prepareUpdate(
    _instance: Instance,
    _type: Type,
    _oldProps: Props,
    newProps: Props,
    _rootContainer: Container,
    _hostContext: HostContext
  ): object | null {
    // TODO: optimize, it always triggers update
    return newProps;
  },

  shouldSetTextContent(_type: Type, _props: Props): boolean {
    return false;
  },
  createTextInstance(
    text: string,
    _rootContainer: Container,
    _hostContext: HostContext,
    _internalHandle: any
  ) {
    return text;
  },

  scheduleTimeout: setTimeout,
  cancelTimeout: clearTimeout,
  noTimeout: -1,
  isPrimaryRenderer: true,
  warnsIfNotActing: true,
  supportsMutation: false,
  supportsPersistence: true,
  supportsHydration: false,

  getInstanceFromNode(_node: any) {
    throw new Error(`getInstanceFromNode not implemented`);
  },

  beforeActiveInstanceBlur() {},
  afterActiveInstanceBlur() {},

  preparePortalMount(_container: Container) {
    throw new Error(`preparePortalMount not implemented`);
  },

  prepareScopeUpdate(_scopeInstance: any, _instance: any) {
    throw new Error(`prepareScopeUpdate not implemented`);
  },

  getInstanceFromScope(_scopeInstance) {
    throw new Error(`getInstanceFromScope not implemented`);
  },

  getCurrentEventPriority(): Reconciler.Lane {
    return DefaultEventPriority;
  },

  detachDeletedInstance(_node: Instance): void {},

  //
  // Persistence methods
  //

  cloneInstance(
    instance: Instance,
    _updatePayload: object | null,
    _type: Type,
    _oldProps: Props,
    newProps: Props,
    _internalInstanceHandle: any,
    keepChildren: boolean,
    _recyclableInstance: Instance | null
  ) {
    const newInstance = new LiveCompositorHostComponent(newProps.props, newProps.sceneBuilder);
    if (keepChildren) {
      newInstance.children = [...instance.children];
      return newInstance;
    } else {
      return newInstance;
    }
  },

  createContainerChildSet(_container: Container): ChildSet {
    return [];
  },
  appendChildToContainerChildSet(childSet: ChildSet, child: Instance | TextInstance) {
    childSet.push(child);
  },
  finalizeContainerChildren(_container: Container, _newChildren: (Instance | TextInstance)[]) {},
  replaceContainerChildren(_container: Container, _newChildren: (Instance | TextInstance)[]) {},

  cloneHiddenInstance(
    _instance: Instance,
    _type: Type,
    props: Props,
    _internalInstanceHandle: any
  ): Instance {
    return new LiveCompositorHostComponent(props.props, props.sceneBuilder);
  },

  cloneHiddenTextInstance(
    _instance: Instance,
    text: string,
    _internalInstanceHandle: any
  ): TextInstance {
    return text;
  },
};

const CompositorRenderer = Reconciler(HostConfig);

class Renderer {
  root: any;
  onUpdateFn: () => void;

  constructor(element: React.ReactElement, onUpdate: () => void, idPrefix: string) {
    const root = CompositorRenderer.createContainer(
      this, // container tag
      LegacyRoot,
      null, // hydrationCallbacks
      false, // isStrictMode
      null, // concurrentUpdatesByDefaultOverride
      idPrefix, // identifierPrefix
      console.error, // onRecoverableError
      null // transitionCallbacks
    );
    this.root = root;
    this.onUpdateFn = onUpdate;

    CompositorRenderer.updateContainer(element, root, null, () => {});
  }

  public scene(): Api.Component {
    // When resetAfterCommit is called `this.root.current` is not updated yet, so we need to rely
    // on `pendingChildren`. I'm not sure it is always populated, so there is a fallback to
    // `root.current`.
    const rootComponent = this.root.pendingChildren[0] ?? rootHostComponent(this.root.current);
    return rootComponent.scene();
  }

  public onUpdate() {
    this.onUpdateFn();
  }
}

function rootHostComponent(root: any): LiveCompositorHostComponent {
  console.error('No pendingChildren found, this might be an error');
  let current = root;
  while (current) {
    if (current?.stateNode instanceof LiveCompositorHostComponent) {
      return current?.stateNode;
    }
    current = current.child;
  }
  throw new Error('No live compositor host component found in the tree.');
}

function groupTextComponents(components: SceneComponent[]): SceneComponent[] {
  const groupedComponents: SceneComponent[] = [];
  let currentString: string | null = null;
  for (const component of components) {
    if (typeof component === 'string') {
      if (currentString === null) {
        currentString = component;
      } else {
        currentString = `${currentString}${component}`;
      }
    } else {
      if (currentString !== null) {
        groupedComponents.push(currentString);
        currentString = null;
      }
      groupedComponents.push(component);
    }
  }
  if (currentString !== null) {
    groupedComponents.push(currentString);
  }

  return groupedComponents;
}

export default Renderer;
