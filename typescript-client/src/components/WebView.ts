import * as Api from '../api';
import { Component } from '../component';
import { intoComponent } from '../element';
import { RenderContext } from '../context';

type WebViewProps = {
  children?: Component<any>[];

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

class WebView extends Component<WebViewProps> {
  props: WebViewProps;

  constructor(props: WebViewProps) {
    super();
    this.props = props;
  }

  scene(ctx: RenderContext): Api.Component {
    return {
      type: 'web_view',
      id: this.props.id,
      instance_id: this.props.instanceId,
      children: (this.props.children || []).map(child => intoComponent(child).scene(ctx)),
    };
  }

  update(props: WebViewProps): void {
    this.props = props;
  }
}

export default WebView;
