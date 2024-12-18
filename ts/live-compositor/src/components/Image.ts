import type { ComponentBaseProps, SceneComponent } from '../component.js';
import { createCompositorComponent } from '../component.js';
import type { Api } from '../index.js';

export type ImageProps = Omit<ComponentBaseProps, 'children'> & {
  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Id of an image. It identifies an image registered using `LiveCompositor.registerImage`.
   */
  imageId: Api.RendererId;
};

const Image = createCompositorComponent<ImageProps>(sceneBuilder);

function sceneBuilder(props: ImageProps, _children: SceneComponent[]): Api.Component {
  return {
    type: 'image',
    id: props.id,
    image_id: props.imageId,
  };
}

export default Image;
