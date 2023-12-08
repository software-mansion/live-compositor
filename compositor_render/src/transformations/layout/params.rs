use compositor_common::util::colors::RGBAColor;
use nalgebra_glm::Mat4;
use wgpu::util::DeviceExt;

use crate::wgpu::WgpuCtx;

#[derive(Debug)]
pub(super) struct LayoutNodeParams {
    pub(super) transform_vertices_matrix: Mat4,
    pub(super) transform_texture_coords_matrix: Mat4,
    pub(super) texture_id: i32,
    pub(super) background_color: RGBAColor,
}

pub(super) struct ParamsBuffer {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    content: bytes::Bytes,
}

impl ParamsBuffer {
    pub fn new(wgpu_ctx: &WgpuCtx, params: Vec<LayoutNodeParams>) -> Self {
        let mut content = Self::shader_buffer_content(&params);
        if content.is_empty() {
            content = bytes::Bytes::copy_from_slice(&[0]);
        }

        let buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: &content,
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

    pub fn update(&mut self, params: Vec<LayoutNodeParams>, wgpu_ctx: &WgpuCtx) {
        let content = Self::shader_buffer_content(&params);
        if self.content.len() != content.len() {
            *self = Self::new(wgpu_ctx, params);
            return;
        }

        if self.content != content {
            wgpu_ctx.queue.write_buffer(&self.buffer, 0, &content);
        }
    }

    fn shader_buffer_content(params: &[LayoutNodeParams]) -> bytes::Bytes {
        params
            .iter()
            .map(LayoutNodeParams::shader_buffer_content)
            .collect::<Vec<[u8; 160]>>()
            .concat()
            .into()
    }
}

impl LayoutNodeParams {
    fn shader_buffer_content(&self) -> [u8; 160] {
        let Self {
            transform_vertices_matrix,
            transform_texture_coords_matrix,
            texture_id,
            background_color,
        } = self;
        let mut result = [0; 160];
        fn from_u8_color(value: u8) -> [u8; 4] {
            (value as f32 / 255.0).to_ne_bytes()
        }

        result[0..64].copy_from_slice(bytemuck::bytes_of(&transform_vertices_matrix.transpose()));
        result[64..128].copy_from_slice(bytemuck::bytes_of(
            &transform_texture_coords_matrix.transpose(),
        ));
        // 12 bytes padding
        result[128..132].copy_from_slice(&from_u8_color(background_color.0));
        result[132..136].copy_from_slice(&from_u8_color(background_color.1));
        result[136..140].copy_from_slice(&from_u8_color(background_color.2));
        result[140..144].copy_from_slice(&from_u8_color(background_color.3));

        result[144..148].copy_from_slice(&texture_id.to_ne_bytes());
        // 12 bytes padding

        result
    }
}
