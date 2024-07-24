import * as Api from '../api';
import { Component } from '../component';

type ImageProps = {
  children: undefined;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId | null;
  /**
   * Id of an image. It identifies an image registered using a [`register image`](../routes.md#register-image) request.
   */
  imageId: Api.RendererId;
};

class Image extends Component<ImageProps> {
  props: ImageProps;

  constructor(props: ImageProps) {
    super();
    this.props = props;
  }

  scene(): Api.Component {
    return {
      type: 'image',
      id: this.props.id,
      image_id: this.props.imageId,
    };
  }

  update(props: ImageProps): void {
    this.props = props;
  }
}

export default Image;
