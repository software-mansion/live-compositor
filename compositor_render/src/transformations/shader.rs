use std::sync::Arc;

use compositor_common::{
    renderer_spec::{FallbackStrategy, ShaderSpec},
    scene::validation::constraints::NodeConstraints,
};

use crate::renderer::WgpuCtx;

use super::shader_executor::{CreateShaderError, ShaderExecutor};

pub mod node;

pub struct Shader {
    executor: ShaderExecutor,
    fallback_strategy: FallbackStrategy,
    clear_color: Option<wgpu::Color>,
    pub constraints: NodeConstraints,
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
}
