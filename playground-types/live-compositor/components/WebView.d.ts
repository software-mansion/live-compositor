import * as Api from '../api';
import LiveCompositorComponent, { SceneBuilder } from '../component';
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
declare class WebView extends LiveCompositorComponent<WebViewProps> {
    builder: SceneBuilder<WebViewProps>;
}
export default WebView;
