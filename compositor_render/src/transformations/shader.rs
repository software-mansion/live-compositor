use std::sync::Arc;

use crate::{
    scene::ShaderParam,
    wgpu::{shader::CreateShaderError, validation::ParametersValidationError, WgpuCtx},
    FallbackStrategy, RendererId,
};

use self::pipeline::ShaderPipeline;

pub mod node;
mod pipeline;

const SHADER_INPUT_TEXTURES_AMOUNT: u32 = 16;

#[derive(Debug)]
pub struct Shader {
    pipeline: ShaderPipeline,
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
        // let wgpu_shader = WgpuShader::new(wgpu_ctx, spec.source)?;
        let pipeline = ShaderPipeline::new(wgpu_ctx, &spec.source)?;

        Ok(Self {
            pipeline,
            fallback_strategy,
            clear_color,
        })
    }

    pub(crate) fn validate_params(
        &self,
        params: &ShaderParam,
    ) -> Result<(), ParametersValidationError> {
        self.pipeline.validate_params(params)
    }
}
