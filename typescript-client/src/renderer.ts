import Reconciler from 'react-reconciler';
import { DefaultEventPriority, LegacyRoot } from 'react-reconciler/constants';
import * as Api from './api';
import { SceneBuilder, SceneComponent } from './component';

export class LiveCompositorHostComponent {
  public props: object;
  public sceneBuilder: SceneBuilder<object>;
  public children: (Instance | TextInstance)[] = [];

  constructor(props: object, sceneBuilder: SceneBuilder<object>) {
    this.props = props;
    this.sceneBuilder = sceneBuilder;
  }

  public scene(children: SceneComponent[]): Api.Component {
    return this.sceneBuilder(this.props, children);
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
  Type, // Type
  Props, // Props
  Container, // Container
  Instance, // Instance
  TextInstance, //TextInstance,
  void, // SuspenseInstance,
  void, // HydratableInstance,
  Instance, //PublicInstance,
  HostContext, //HostContext,
  object, // UpdatePayload,
  ChildSet, // ChildSet,
  NodeJS.Timeout, // TimeoutHandle,
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

  resetAfterCommit(_containerInfo: Container): void {},

  createInstance(
    type: Type,
    props: Props,
    _rootContainer: Container,
    _hostContext: HostContext,
    _internalHandle: any
  ): LiveCompositorHostComponent {
    //console.log('this.createInstance', type, props, rootContainer, hostContext, internalHandle);
    //console.log('this.createInstance', type, props.scene);
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
    _instance: Instance,
    _updatePayload: object | null,
    _type: Type,
    _oldProps: Props,
    newProps: Props,
    _internalInstanceHandle: any,
    _keepChildren: boolean,
    _recyclableInstance: Instance | null
  ) {
    return new LiveCompositorHostComponent(newProps.props, newProps.sceneBuilder);
  },

  createContainerChildSet(_container: Container) {
    return [];
  },
  appendChildToContainerChildSet(childSet: ChildSet, child: Instance | TextInstance) {
    childSet.push(child);
  },
  finalizeContainerChildren(_container: Container, _newChildren: (Instance | TextInstance)[]) {},
  replaceContainerChildren(container: Container, _newChildren: (Instance | TextInstance)[]) {
    container.onUpdate();
  },

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
  onUpdateFn: (scene: SceneComponent[]) => void;

  constructor(element: React.ReactElement, onUpdate: (scene: SceneComponent[]) => void) {
    const root = CompositorRenderer.createContainer(
      this, // container tag
      LegacyRoot,
      null, // hydrationCallbacks
      false, // isStrictMode
      null, // concurrentUpdatesByDefaultOverride
      '', // identifierPrefix
      console.error, // onRecoverableError
      null //transitionCallbacks
    );
    this.root = root;
    this.onUpdateFn = onUpdate;

    CompositorRenderer.updateContainer(element, root, null, () => {});
  }

  public scene(): SceneComponent[] {
    return buildScene(this.root.current);
  }

  public onUpdate() {
    this.onUpdateFn(this.scene());
  }
}

function buildScene(root: any): SceneComponent[] {
  const components: SceneComponent[] = [];

  let current = root;
  while (current) {
    const stateNode = current?.stateNode;
    const children = current.child ? buildScene(current.child) : [];
    if (stateNode instanceof LiveCompositorHostComponent) {
      components.push(stateNode.scene(children));
    } else if (typeof stateNode === 'string') {
      components.push(stateNode);
    } else {
      components.push(...children);
    }

    current = current.sibling;
  }

  // concat raw strings
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
