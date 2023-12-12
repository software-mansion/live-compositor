use std::sync::Arc;

use crate::{
    scene::ShaderParam,
    wgpu::{
        shader::{CreateShaderError, WgpuShader},
        validation::ParametersValidationError,
        WgpuCtx,
    },
    FallbackStrategy, RendererId,
};

pub mod node;

#[derive(Debug)]
pub struct Shader {
    wgpu_shader: WgpuShader,
    fallback_strategy: FallbackStrategy,
    clear_color: Option<wgpu::Color>,
}

#[derive(Debug)]
pub struct ShaderSpec {
    pub shader_id: RendererId,
    pub source: String,
    pub fallback_strategy: FallbackStrategy,
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
