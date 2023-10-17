use std::sync::Arc;

use compositor_common::{
    renderer_spec::{FallbackStrategy, ShaderSpec},
    scene::constraints::NodeConstraints,
};

use crate::wgpu::{
    shader::{CreateShaderError, WgpuShader},
    WgpuCtx,
};

pub mod node;

pub struct Shader {
    wgpu_shader: WgpuShader,
    fallback_strategy: FallbackStrategy,
    clear_color: Option<wgpu::Color>,
    constraints: NodeConstraints,
}

impl Shader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>, spec: ShaderSpec) -> Result<Self, CreateShaderError> {
        let fallback_strategy = spec.fallback_strategy;
        let clear_color = None;
        let constraints = spec.constraints;
        let wgpu_shader = WgpuShader::new(wgpu_ctx, spec.source)?;

        Ok(Self {
            wgpu_shader,
            fallback_strategy,
            clear_color,
            constraints,
        })
    }

    pub fn constraints(&self) -> &NodeConstraints {
        &self.constraints
    }
}
