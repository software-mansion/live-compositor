import * as Api from '../api';
import { Component } from '../component';

type InputStreamProps = {
  children: undefined;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Id of an input. It identifies a stream registered using a [`RegisterInputStream`](../routes.md#register-input) request.
   */
  inputId: Api.InputId;
};

class InputStream extends Component<InputStreamProps> {
  props: InputStreamProps;

  constructor(props: InputStreamProps) {
    super();
    this.props = props;
  }

  scene(): Api.Component {
    return {
      type: 'input_stream',
      id: this.props.id,
      input_id: this.props.inputId,
    };
  }

  update(props: InputStreamProps): void {
    this.props = props;
  }
}

export default InputStream;
