use std::sync::Arc;

use crate::{
    scene::ShaderParam,
    wgpu::{common_pipeline::CreateShaderError, WgpuCtx},
    FallbackStrategy, RendererId,
};

use self::{pipeline::ShaderPipeline, validation::error::ParametersValidationError};

mod base_params;
pub mod node;
mod pipeline;
pub mod validation;

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
