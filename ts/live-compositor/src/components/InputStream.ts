import * as Api from '../api';
import LiveCompositorComponent, { SceneBuilder, SceneComponent } from '../component';

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

class InputStream extends LiveCompositorComponent<InputStreamProps> {
  builder: SceneBuilder<InputStreamProps> = sceneBuilder;
}

function sceneBuilder(props: InputStreamProps, _children: SceneComponent[]): Api.Component {
  return {
    type: 'input_stream',
    id: props.id,
    input_id: props.inputId,
  };
}

export default InputStream;
