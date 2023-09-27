use crate::renderer::WgpuCtx;
use wgpu::util::DeviceExt;

pub(crate) struct ParamsBuffer {
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    content: bytes::Bytes,
}

impl ParamsBuffer {
    pub fn new(content: bytes::Bytes, wgpu_ctx: &WgpuCtx) -> Self {
        let content_or_zero = match content.is_empty() {
            true => bytes::Bytes::copy_from_slice(&[0]),
            false => content.clone(),
        };

        let buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: &content_or_zero,
            });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("params bind group"),
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

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
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
