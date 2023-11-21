use compositor_common::scene::shader;
use compositor_common::scene::transition;

use super::component::*;

impl From<shader::ShaderParam> for ShaderParam {
    fn from(param: shader::ShaderParam) -> Self {
        fn from_struct_field(field: shader::ShaderParamStructField) -> ShaderParamStructField {
            ShaderParamStructField {
                field_name: field.field_name,
                value: field.value.into(),
            }
        }
        match param {
            shader::ShaderParam::F32(value) => ShaderParam::F32(value),
            shader::ShaderParam::U32(value) => ShaderParam::U32(value),
            shader::ShaderParam::I32(value) => ShaderParam::I32(value),
            shader::ShaderParam::List(value) => {
                ShaderParam::List(value.into_iter().map(Into::into).collect())
            }
            shader::ShaderParam::Struct(value) => {
                ShaderParam::Struct(value.into_iter().map(from_struct_field).collect())
            }
        }
    }
}

impl From<transition::Interpolation> for Interpolation {
    fn from(interpolation: transition::Interpolation) -> Self {
        match interpolation {
            transition::Interpolation::Linear => Interpolation::Linear,
            transition::Interpolation::Spring => Interpolation::Spring,
        }
    }
}
