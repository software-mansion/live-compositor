import * as Api from '../api.js';
import { createCompositorComponent, SceneComponent } from '../component.js';

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

const Image = createCompositorComponent<ImageProps>(sceneBuilder);

function sceneBuilder(props: ImageProps, _children: SceneComponent[]): Api.Component {
  const x = '1234';
  return {
    type: 'image',
    id: props.id,
    image_id: props.imageId,
  };
}

export default Image;
