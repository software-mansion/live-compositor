import * as Api from '../api';
import LiveCompositorComponent, { SceneBuilder } from '../component';
export type ImageProps = {
    children?: undefined;
    /**
     * Id of a component.
     */
    id?: Api.ComponentId | null;
    /**
     * Id of an image. It identifies an image registered using `LiveCompositor.registerImage`.
     */
    imageId: Api.RendererId;
};
declare class Image extends LiveCompositorComponent<ImageProps> {
    builder: SceneBuilder<ImageProps>;
}
export default Image;
