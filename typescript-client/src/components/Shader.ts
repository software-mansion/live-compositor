import * as Api from '../api';
import { Component } from '../component';
import { RenderContext } from '../context';
import { intoComponent } from '../element';

type ShaderProps = {
  children: Component<any>[];

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

class Shader extends Component<ShaderProps> {
  props: ShaderProps;

  constructor(props: ShaderProps) {
    super();
    this.props = props;
  }

  scene(ctx: RenderContext): Api.Component {
    return {
      type: 'shader',
      id: this.props.id,
      shader_id: this.props.shaderId,
      shader_param: this.props.shaderParam, // TODO: map from snake case
      resolution: this.props.resolution,
      children: this.props.children.map(c => intoComponent(c).scene(ctx)) || [],
    };
  }

  update(props: ShaderProps): void {
    this.props = props;
  }
}

export default Shader;
