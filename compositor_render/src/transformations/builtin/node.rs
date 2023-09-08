use std::{sync::Arc, time::Duration};

use compositor_common::{
    renderer_spec::FallbackStrategy,
    scene::{NodeId, Resolution},
};

use wgpu::util::DeviceExt;

use crate::{
    renderer::{texture::NodeTexture, WgpuCtx},
    transformations::shader::Shader,
};

use super::{params::BuiltinParams, Builtin};

struct ParamsBuffer {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    content: bytes::Bytes,
}

impl ParamsBuffer {
    pub fn new(mut content: bytes::Bytes, wgpu_ctx: &WgpuCtx) -> Self {
        if content.is_empty() {
            content = bytes::Bytes::copy_from_slice(&[0]);
        }

        let buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("builtin node params buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: &content,
            });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("builtin node params bind group"),
                layout: &wgpu_ctx.shader_parameters_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            bind_group,
            buffer,
            content,
        }
    }

    pub fn update(&mut self, content: bytes::Bytes, wgpu_ctx: &WgpuCtx) {
        if self.content.len() != content.len() {
            *self = Self::new(content, wgpu_ctx);
            return;
        }

        if self.content != content {
            wgpu_ctx.queue.write_buffer(&self.buffer, 0, &content);
        }
    }
}

pub struct BuiltinNode {
    shader: Arc<Shader>,
    builtin: Builtin,
    params_buffer: ParamsBuffer,
    clear_color: Option<wgpu::Color>,
}

impl BuiltinNode {
    pub fn new(shader: Arc<Shader>, builtin: Builtin, input_count: u32) -> Self {
        let input_resolutions = vec![None; input_count as usize];

        let params_buffer_content =
            BuiltinParams::new(&builtin, &input_resolutions).shader_buffer_content();
        let params_buffer = ParamsBuffer::new(params_buffer_content, &shader.wgpu_ctx);

        let clear_color = builtin.clear_color();

        Self {
            shader,
            builtin,
            params_buffer,
            clear_color,
        }
    }

    // Returns Some(Resolution) if output resolution of node can be determined
    // from spec (on scene update). If output resolution is depended on input resolutions,
    // then returns None.
    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        self.builtin.resolution_from_spec()
    }

    pub fn render(
        &mut self,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        let input_resolutions: Vec<Option<Resolution>> = sources
            .iter()
            .map(|(_, node_texture)| node_texture.resolution())
            .collect();

        let output_resolution = self.builtin.output_resolution(&input_resolutions);

        let params_buffer_content =
            BuiltinParams::new(&self.builtin, &input_resolutions).shader_buffer_content();

        self.params_buffer
            .update(params_buffer_content, &self.shader.wgpu_ctx);

        let target = target.ensure_size(&self.shader.wgpu_ctx, output_resolution);
        self.shader.render(
            &self.params_buffer.bind_group,
            sources,
            target,
            pts,
            self.clear_color,
        );
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        self.shader.fallback_strategy
    }
}
