use bytes::Bytes;
use nalgebra_glm::Mat4;
use wgpu::util::DeviceExt;

use crate::{scene::RGBAColor, wgpu::WgpuCtx, Resolution};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(super) enum RoundingDirection {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone)]
pub(super) struct RoundedCorner {
    pub radius: f32,
    pub center: (f32, f32),
    pub direction: RoundingDirection,
}

#[derive(Debug, Clone)]
pub(super) struct LayoutNodeParams {
    pub(super) transform_vertices_matrix: Mat4,
    pub(super) transform_texture_coords_matrix: Mat4,
    pub(super) is_texture: u32,
    pub(super) background_color: RGBAColor,
    pub(super) rounded_corners: Vec<RoundedCorner>,
}

struct Buffer {
    buffer: wgpu::Buffer,
    content: Bytes,
}

impl Buffer {
    pub fn new(wgpu_ctx: &WgpuCtx, mut content: Bytes) -> Self {
        // wgpu panics when creating bind group with empty buffer
        if content.is_empty() {
            content = Bytes::from_static(&[0]);
        }
        let buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params buffer"),
                contents: &content,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        Self { buffer, content }
    }

    pub fn update(&mut self, wgpu_ctx: &WgpuCtx, mut content: Bytes) {
        // wgpu panics when creating bind group with empty buffer
        if content.is_empty() {
            content = Bytes::from_static(&[0]);
        }

        if self.content.len() != content.len() {
            *self = Self::new(wgpu_ctx, content);
            return;
        }

        if self.content != content {
            wgpu_ctx.queue.write_buffer(&self.buffer, 0, &content);
            self.content = content;
        }
    }
}

pub(super) struct LayoutParamBuffers {
    bind_group_1_buffer: Buffer,
    bind_group_2_buffers: Vec<BindGroup2Buffers>,
    bind_group_2_layout: wgpu::BindGroupLayout,
}

struct BindGroup2Buffers {
    rounded_corners_buffer: Buffer,
    layouts_info_buffer: Buffer,
}

impl BindGroup2Buffers {
    pub fn new(
        wgpu_ctx: &WgpuCtx,
        params: &LayoutNodeParams,
        layout_id: u32,
        output_resolution: Resolution,
    ) -> Self {
        let rounded_corners_buffer = Buffer::new(
            wgpu_ctx,
            rounded_corners_buffer_content(&params.rounded_corners),
        );
        let layouts_info_buffer = Buffer::new(
            wgpu_ctx,
            layout_info_buffer_content(
                layout_id,
                params.rounded_corners.len() as u32,
                output_resolution,
            ),
        );
        Self {
            rounded_corners_buffer,
            layouts_info_buffer,
        }
    }

    pub fn update(
        &mut self,
        wgpu_ctx: &WgpuCtx,
        params: &LayoutNodeParams,
        layout_id: u32,
        output_resolution: Resolution,
    ) {
        self.rounded_corners_buffer.update(
            wgpu_ctx,
            rounded_corners_buffer_content(&params.rounded_corners),
        );
        self.layouts_info_buffer.update(
            wgpu_ctx,
            layout_info_buffer_content(
                layout_id,
                params.rounded_corners.len() as u32,
                output_resolution,
            ),
        );
    }
}

impl LayoutParamBuffers {
    pub fn new(
        wgpu_ctx: &WgpuCtx,
        params: &[LayoutNodeParams],
        output_resolution: Resolution,
    ) -> Self {
        let bind_group_1_buffer = Buffer::new(wgpu_ctx, layouts_buffer_content(params));
        let bind_group_2_buffers = params
            .iter()
            .enumerate()
            .map(|(layout_id, params)| {
                BindGroup2Buffers::new(wgpu_ctx, params, layout_id as u32, output_resolution)
            })
            .collect();

        let bind_group_2_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind group 2 layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        Self {
            bind_group_1_buffer,
            bind_group_2_buffers,
            bind_group_2_layout,
        }
    }

    pub fn create_bind_groups(&self, wgpu_ctx: &WgpuCtx) -> LayoutBindGroups {
        let bind_group_1 = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &wgpu_ctx.uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        self.bind_group_1_buffer.buffer.as_entire_buffer_binding(),
                    ),
                }],
                label: Some("Layouts bind group"),
            });

        let bind_groups_2 = self
            .bind_group_2_buffers
            .iter()
            .map(|buffers| {
                wgpu_ctx
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.bind_group_2_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(
                                    buffers
                                        .rounded_corners_buffer
                                        .buffer
                                        .as_entire_buffer_binding(),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(
                                    buffers
                                        .layouts_info_buffer
                                        .buffer
                                        .as_entire_buffer_binding(),
                                ),
                            },
                        ],
                        label: Some("Layouts bind group 2"),
                    })
            })
            .collect::<Vec<_>>();

        LayoutBindGroups {
            bind_group_1,
            bind_groups_2,
        }
    }

    pub fn update_buffers(
        &mut self,
        wgpu_ctx: &WgpuCtx,
        params: &[LayoutNodeParams],
        output_resolution: Resolution,
    ) {
        self.bind_group_1_buffer
            .update(wgpu_ctx, layouts_buffer_content(params));

        for (layout_id, params) in params.iter().enumerate() {
            match self.bind_group_2_buffers.get_mut(layout_id) {
                Some(buffer) => {
                    buffer.update(wgpu_ctx, params, layout_id as u32, output_resolution);
                }
                None => {
                    self.bind_group_2_buffers.push(BindGroup2Buffers::new(
                        wgpu_ctx,
                        params,
                        layout_id as u32,
                        output_resolution,
                    ));
                }
            }
        }
    }
}

pub(super) struct LayoutBindGroups {
    pub bind_group_1: wgpu::BindGroup,
    pub bind_groups_2: Vec<wgpu::BindGroup>,
}

fn layouts_buffer_content(params: &[LayoutNodeParams]) -> bytes::Bytes {
    params
        .iter()
        .map(|params| {
            let mut result = [0; 160];
            fn from_u8_color(value: u8) -> [u8; 4] {
                (value as f32 / 255.0).to_ne_bytes()
            }

            result[0..64].copy_from_slice(bytemuck::bytes_of(
                &params.transform_vertices_matrix.transpose(),
            ));
            result[64..128].copy_from_slice(bytemuck::bytes_of(
                &params.transform_texture_coords_matrix.transpose(),
            ));
            result[128..132].copy_from_slice(&from_u8_color(params.background_color.0));
            result[132..136].copy_from_slice(&from_u8_color(params.background_color.1));
            result[136..140].copy_from_slice(&from_u8_color(params.background_color.2));
            result[140..144].copy_from_slice(&from_u8_color(params.background_color.3));

            result[144..148].copy_from_slice(&params.is_texture.to_ne_bytes());
            // 12 bytes padding

            result
        })
        .collect::<Vec<[u8; 160]>>()
        .concat()
        .into()
}

fn rounded_corners_buffer_content(corners: &[RoundedCorner]) -> Bytes {
    if corners.is_empty() {
        return Bytes::from_static(&[0; 16]);
    }
    corners
        .iter()
        .map(|corner| {
            let mut result = [0; 16];
            result[0..4].copy_from_slice(&corner.center.0.to_ne_bytes());
            result[4..8].copy_from_slice(&corner.center.1.to_ne_bytes());
            result[8..12].copy_from_slice(&corner.radius.to_ne_bytes());
            result[12..16].copy_from_slice(
                &match corner.direction {
                    RoundingDirection::TopLeft => 0u32,
                    RoundingDirection::TopRight => 1u32,
                    RoundingDirection::BottomRight => 2u32,
                    RoundingDirection::BottomLeft => 3u32,
                }
                .to_ne_bytes(),
            );
            result
        })
        .collect::<Vec<[u8; 16]>>()
        .concat()
        .into()
}

fn layout_info_buffer_content(
    layout_id: u32,
    rounded_corners_count: u32,
    output_resolution: Resolution,
) -> Bytes {
    let mut result = [0; 16];
    result[0..4].copy_from_slice(&layout_id.to_ne_bytes());
    result[4..8].copy_from_slice(&rounded_corners_count.to_ne_bytes());
    result[8..12].copy_from_slice(&(output_resolution.width as f32).to_ne_bytes());
    result[12..16].copy_from_slice(&(output_resolution.height as f32).to_ne_bytes());
    Bytes::copy_from_slice(&result)
}
