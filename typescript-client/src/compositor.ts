import * as ApiTypes from './api';
import Api from './api';
import { ClassComponent, Component, FunctionComponent } from './component';
import { createElement, Element } from './element';
import { LiveCompositorState } from './eventLoop';
import { ServerManager } from './severInstance';
import ManagedInstance from './severInstance/managed';
import { Context } from './types';

class LiveCompositor<Props> {
  private root: Component<Props>;
  private ctx: Context = {
    inputs: [],
  };
  private state: LiveCompositorState = new LiveCompositorState();
  private lastScene: ApiTypes.Component | null = null;
  private serverManager: ServerManager;
  private apiInstance: Api;

  constructor(root: Element<Props>, serverManager?: ServerManager) {
    if (typeof root === 'string') {
      throw new Error("root component can't be a string");
    }
    this.root = root as Component<Props>;
    this.serverManager = serverManager ?? new ManagedInstance(8000, '/tmp/compositor_tmp');
    this.apiInstance = new Api(this.serverManager);
  }

  public api(): Api {
    return this.apiInstance;
  }

  public update(props: Props) {
    this.root.update(props);
    this.render();
  }

  public static createElement<Props>(
    component: FunctionComponent<Props> | ClassComponent<Props>,
    props: Props,
    ...children: Element<any>[]
  ): Element<Props> {
    return createElement(component, props, ...children);
  }

  public render() {
    const { renderContext, contextDone } = this.state.initRenderContext(this.ctx);
    const scene = this.root.scene(renderContext);
    contextDone();
    if (JSON.stringify(scene) !== JSON.stringify(this.lastScene)) {
      this.lastScene = scene;
    }
    this.state.maybeScheduleNextUpdate(this);
  }

  public async start() {
    return this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/${encodeURIComponent(inputId)}/unregister`,
      body: {},
    });
    this.state.start();
    this.render();
  }
}

export default LiveCompositor;
