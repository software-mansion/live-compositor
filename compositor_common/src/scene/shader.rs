use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case", content = "value")]
pub enum ShaderParam {
    F32(f32),
    U32(u32),
    I32(i32),
    List(Vec<ShaderParam>),
    Struct(Vec<ShaderParamStructField>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderParamStructField {
    pub field_name: String,
    #[serde(flatten)]
    pub value: ShaderParam,
}

impl From<(&'static str, ShaderParam)> for ShaderParamStructField {
    fn from(value: (&'static str, ShaderParam)) -> Self {
        Self {
            field_name: value.0.to_owned(),
            value: value.1,
        }
    }
}
