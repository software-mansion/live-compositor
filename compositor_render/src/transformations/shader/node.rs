use std::{sync::Arc, time::Duration};

use compositor_common::{
    renderer_spec::{FallbackStrategy, RendererId},
    scene::{shader::ShaderParam, NodeId, Resolution},
};
use wgpu::util::DeviceExt;

use crate::renderer::texture::utils::sources_to_textures;
use crate::{
    error::CreateNodeError,
    renderer::{texture::NodeTexture, RenderCtx, WgpuCtx},
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
        ctx: &RenderCtx,
        shader_id: &RendererId,
        shader_params: &Option<ShaderParam>,
        resolution: &Resolution,
    ) -> Result<Self, CreateNodeError> {
        let shader = ctx
            .renderers
            .shaders
            .get(shader_id)
            .ok_or_else(|| CreateNodeError::ShaderNotFound(shader_id.clone()))?;

        if let Some(params) = shader_params {
            shader.gpu_shader.validate_params(params).map_err(|err| {
                CreateNodeError::ShaderNodeParametersValidationError(err, shader_id.clone())
            })?
        }

        let custom_params_buffer = Self::new_params_buffer(ctx.wgpu_ctx, shader_params);
        let params_bind_group = Self::new_params_bind_group(ctx.wgpu_ctx, &custom_params_buffer);

        Ok(Self {
            params_bind_group,
            _custom_params_buffer: custom_params_buffer,
            shader,
            resolution: *resolution,
        })
    }

    fn new_params_buffer(ctx: &WgpuCtx, shader_params: &Option<ShaderParam>) -> wgpu::Buffer {
        match shader_params {
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
        }
    }

    fn new_params_bind_group(ctx: &WgpuCtx, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shader node params bind group"),
            layout: &ctx.shader_parameters_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
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
        let target = target.ensure_size(&self.shader.gpu_shader.wgpu_ctx, self.resolution);
        let textures = sources_to_textures(sources);
        self.shader.gpu_shader.render(
            &self.params_bind_group,
            &textures,
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
