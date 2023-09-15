use std::sync::Arc;

use compositor_common::{
    renderer_spec::{FallbackStrategy, ShaderSpec},
    scene::constraints::NodeConstraints,
};

use crate::{
    gpu_shader::{CreateShaderError, GpuShader},
    renderer::WgpuCtx,
};

pub mod node;

pub struct Shader {
    gpu_shader: GpuShader,
    fallback_strategy: FallbackStrategy,
    clear_color: Option<wgpu::Color>,
    constraints: NodeConstraints,
}

impl Shader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>, spec: ShaderSpec) -> Result<Self, CreateShaderError> {
        let fallback_strategy = spec.fallback_strategy;
        let clear_color = None;
        let constraints = spec.constraints;
        let gpu_shader = GpuShader::new(wgpu_ctx, spec.source)?;

        Ok(Self {
            gpu_shader,
            fallback_strategy,
            clear_color,
            constraints,
        })
    }

    pub fn constraints(&self) -> &NodeConstraints {
        &self.constraints
    }
}
