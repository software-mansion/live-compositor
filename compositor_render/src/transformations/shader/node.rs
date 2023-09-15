use std::{sync::Arc, time::Duration};

use compositor_common::{
    renderer_spec::FallbackStrategy,
    scene::{shader::ShaderParam, NodeId, Resolution},
};
use wgpu::util::DeviceExt;

use crate::{
    renderer::{texture::NodeTexture, WgpuCtx},
    shader_executor::error::ParametersValidationError,
};

use super::Shader;

pub struct ShaderNode {
    params_bind_group: wgpu::BindGroup,
    _custom_params_buffer: wgpu::Buffer,
    shader: Arc<Shader>,
    resolution: Resolution,
}

impl ShaderNode {
    pub fn new(
        ctx: &WgpuCtx,
        shader: Arc<Shader>,
        params: Option<&ShaderParam>,
        resolution: Resolution,
    ) -> Result<Self, ParametersValidationError> {
        if let Some(params) = params {
            shader.executor.validate_params(params)?;
        }

        let custom_params_buffer = match params {
            Some(params) => {
                let params = params.to_bytes();
                ctx.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("shader node custom params buffer"),
                        usage: wgpu::BufferUsages::UNIFORM,
                        contents: &params,
                    })
            }
            None => ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("shader node empty custom params buffer"),
                    contents: &[0],
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
        };

        let params_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shader node params bind group"),
            layout: &ctx.shader_parameters_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: custom_params_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            params_bind_group,
            _custom_params_buffer: custom_params_buffer,
            shader,
            resolution,
        })
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        self.shader.fallback_strategy
    }

    pub fn render(
        &self,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        let target = target.ensure_size(&self.shader.executor.wgpu_ctx, self.resolution);
        self.shader.executor.render(
            &self.params_bind_group,
            sources,
            target,
            pts,
            self.shader.clear_color,
        )
    }
}

pub trait ShaderParamExt {
    fn to_bytes(&self) -> bytes::Bytes;
}

impl ShaderParamExt for ShaderParam {
    fn to_bytes(&self) -> bytes::Bytes {
        fn extend_bytes(s: &ShaderParam, bytes: &mut bytes::BytesMut) {
            match s {
                ShaderParam::F32(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                ShaderParam::U32(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                ShaderParam::I32(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                ShaderParam::List(v) => v.iter().for_each(|v| extend_bytes(v, bytes)),
                ShaderParam::Struct(v) => v.iter().for_each(|v| extend_bytes(&v.value, bytes)),
            }
        }

        let mut result = bytes::BytesMut::new();

        extend_bytes(self, &mut result);

        result.freeze()
    }
}
