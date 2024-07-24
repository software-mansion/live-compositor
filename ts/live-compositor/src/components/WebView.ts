import * as Api from '../api';
import LiveCompositorComponent, {
  SceneBuilder,
  SceneComponent,
  sceneComponentIntoApi,
} from '../component';

type WebViewProps = {
  /**
   * Id of a component.
   */
  id?: Api.ComponentId | null;
  /**
   * Id of a web renderer instance. It identifies an instance registered using a [`register web renderer`](../routes.md#register-web-renderer-instance) request.
   *
   * You can only refer to specific instances in one Component at a time.
   */
  instanceId: Api.RendererId;
};

class WebView extends LiveCompositorComponent<WebViewProps> {
  builder: SceneBuilder<WebViewProps> = sceneBuilder;
}

function sceneBuilder(props: WebViewProps, children: SceneComponent[]): Api.Component {
  return {
    type: 'web_view',
    children: children.map(sceneComponentIntoApi),
    id: props.id,
    instance_id: props.instanceId,
  };
}

export default WebView;
