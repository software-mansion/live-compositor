#[derive(Debug, Clone)]
pub enum ShaderParam {
    F32(f32),
    U32(u32),
    I32(i32),
    List(Vec<ShaderParam>),
    Struct(Vec<ShaderParamStructField>),
}

#[derive(Debug, Clone)]
pub struct ShaderParamStructField {
    pub field_name: String,
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
