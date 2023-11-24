use std::sync::Arc;

use compositor_common::{
    renderer_spec::{FallbackStrategy, ShaderSpec},
    scene::shader::ShaderParam,
};

use crate::wgpu::{
    shader::{CreateShaderError, WgpuShader},
    validation::ParametersValidationError,
    WgpuCtx,
};

pub mod node;

#[derive(Debug)]
pub struct Shader {
    wgpu_shader: WgpuShader,
    fallback_strategy: FallbackStrategy,
    clear_color: Option<wgpu::Color>,
}

impl Shader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>, spec: ShaderSpec) -> Result<Self, CreateShaderError> {
        let fallback_strategy = spec.fallback_strategy;
        let clear_color = None;
        let wgpu_shader = WgpuShader::new(wgpu_ctx, spec.source)?;

        Ok(Self {
            wgpu_shader,
            fallback_strategy,
            clear_color,
        })
    }

    pub(crate) fn validate_params(
        &self,
        params: &ShaderParam,
    ) -> Result<(), ParametersValidationError> {
        self.wgpu_shader.validate_params(params)
    }
}
