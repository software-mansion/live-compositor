use std::sync::Arc;

use compositor_common::{
    renderer_spec::{FallbackStrategy, ShaderSpec},
    scene::constraints::Constraints,
};

use crate::renderer::WgpuCtx;

use super::shader_executor::{CreateShaderError, ShaderExecutor};

pub mod node;

pub struct Shader {
    executor: ShaderExecutor,
    fallback_strategy: FallbackStrategy,
    clear_color: Option<wgpu::Color>,
    constraints: Constraints,
}

impl Shader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>, spec: ShaderSpec) -> Result<Self, CreateShaderError> {
        let fallback_strategy = spec.fallback_strategy;
        let clear_color = None;
        let constraints = spec.constraints;
        let executor = ShaderExecutor::new(wgpu_ctx, spec.source)?;

        Ok(Self {
            executor,
            fallback_strategy,
            clear_color,
            constraints,
        })
    }

    pub fn constraints(&self) -> &Constraints {
        &self.constraints
    }
}
