import * as Api from '../api.js';
import { createCompositorComponent, SceneComponent, sceneComponentIntoApi } from '../component.js';

export type WebViewProps = {
  /**
   * Id of a component.
   */
  id?: Api.ComponentId | null;
  /**
   * Id of a web renderer instance. It identifies an instance registered using `LiveCompositor.registerWebRenderer`.
   *
   * You can only refer to specific instances in one Component at a time.
   */
  instanceId: Api.RendererId;
};

const WebView = createCompositorComponent<WebViewProps>(sceneBuilder);

function sceneBuilder(props: WebViewProps, children: SceneComponent[]): Api.Component {
  return {
    type: 'web_view',
    children: children.map(sceneComponentIntoApi),
    id: props.id,
    instance_id: props.instanceId,
  };
}

export default WebView;
