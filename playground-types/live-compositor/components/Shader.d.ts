import * as Api from '../api';
import LiveCompositorComponent, { SceneBuilder } from '../component';
export type ShaderProps = {
    /**
     * Id of a component.
     */
    id?: Api.ComponentId;
    /**
     * Id of a shader. It identifies a shader registered using `LiveCompositor.registerShader`.
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
    shaderParam?: ShaderParam;
    /**
     * Resolution of a texture where shader will be executed.
     */
    resolution: Api.Resolution;
};
export type ShaderParam = {
    type: 'f32';
    value: number;
} | {
    type: 'u32';
    value: number;
} | {
    type: 'i32';
    value: number;
} | {
    type: 'list';
    value: ShaderParam[];
} | {
    type: 'struct';
    value: ShaderParamStructField[];
};
export type ShaderParamStructField = {
    type: 'f32';
    value: number;
    fieldName: string;
} | {
    type: 'u32';
    value: number;
    fieldName: string;
} | {
    type: 'i32';
    value: number;
    fieldName: string;
} | {
    type: 'list';
    value: ShaderParam[];
    fieldName: string;
} | {
    type: 'struct';
    value: ShaderParamStructField[];
    fieldName: string;
};
declare class Shader extends LiveCompositorComponent<ShaderProps> {
    builder: SceneBuilder<ShaderProps>;
}
export default Shader;
