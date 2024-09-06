import * as Api from '../api';
import LiveCompositorComponent, { SceneBuilder } from '../component';
export type InputStreamProps = {
    children?: undefined;
    /**
     * Id of a component.
     */
    id?: Api.ComponentId;
    /**
     * Id of an input. It identifies a stream registered using a `LiveCompositor.registerInput`.
     */
    inputId: Api.InputId;
};
declare class InputStream extends LiveCompositorComponent<InputStreamProps> {
    builder: SceneBuilder<InputStreamProps>;
}
export default InputStream;
