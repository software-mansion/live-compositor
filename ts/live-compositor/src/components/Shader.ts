import * as Api from '../api';
import LiveCompositorComponent, {
  SceneBuilder,
  SceneComponent,
  sceneComponentIntoApi,
} from '../component';

type ShaderProps = {
  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Id of a shader. It identifies a shader registered using a [`register shader`](../routes.md#register-shader) request.
   */
  shaderId: Api.RendererId;
  /**
   * Object that will be serialized into a `struct` and passed inside the shader as:
   *
   * ```wgsl
   * @group(1) @binding(0) var<uniform>
   * ```
   * :::note
   * This object's structure must match the structure defined in a shader source code. Currently, we do not handle memory layout automatically.
   * To achieve the correct memory alignment, you might need to pad your data with additional fields. See [WGSL documentation](https://www.w3.org/TR/WGSL/#alignment-and-size) for more details.
   * :::
   */
  shaderParam?: Api.ShaderParam;
  /**
   * Resolution of a texture where shader will be executed.
   */
  resolution: Api.Resolution;
};

class Shader extends LiveCompositorComponent<ShaderProps> {
  builder: SceneBuilder<ShaderProps> = sceneBuilder;
}

function sceneBuilder(props: ShaderProps, children: SceneComponent[]): Api.Component {
  return {
    type: 'shader',
    children: children.map(sceneComponentIntoApi),
    id: props.id,
    shader_id: props.shaderId,
    shader_param: props.shaderParam, // TODO: map from snake case
    resolution: props.resolution,
  };
}

export default Shader;
