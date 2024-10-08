use std::alloc::Layout;

use nalgebra_glm::Mat4;
use wgpu::util::DeviceExt;

use crate::{scene::RGBAColor, transformations::layout::RenderLayoutContent, wgpu::WgpuCtx};

use super::RenderLayout;

#[derive(Debug, Clone)]
pub(super) struct LayoutNodeParams {
    pub(super) transform_vertices_matrix: Mat4,
    pub(super) transform_texture_coords_matrix: Mat4,
    pub(super) is_texture: u32,
    pub(super) background_color: RGBAColor,
}

impl Default for LayoutNodeParams {
    fn default() -> Self {
        Self {
            transform_vertices_matrix: Mat4::identity(),
            transform_texture_coords_matrix: Mat4::identity(),
            is_texture: 0,
            background_color: RGBAColor(0, 0, 0, 0),
        }
    }
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
                layout: &wgpu_ctx.uniform_bgl,
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
        }

        if self.content != content {
            wgpu_ctx.queue.write_buffer(&self.buffer, 0, &content);
        }
    }

    fn shader_buffer_content(params: &[LayoutNodeParams]) -> bytes::Bytes {
        // this should only be enabled on `wasm32`, but it needs to be enabled as a temporary fix
        // (@wbarczynski has a PR fixing this in the works right now)
        let params = {
            // On WebGL we have to fill the whole array
            const MAX_PARAMS_COUNT: usize = 100;
            let mut params = params.to_vec();
            params.resize_with(MAX_PARAMS_COUNT, LayoutNodeParams::default);
            params
        };

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
            is_texture,
            background_color,
        } = self;
        let mut result = [0; 160];
        fn from_u8_color(value: u8) -> [u8; 4] {
            (value as f32 / 255.0).to_le_bytes()
        }

        result[0..64].copy_from_slice(bytemuck::bytes_of(&transform_vertices_matrix.transpose()));
        result[64..128].copy_from_slice(bytemuck::bytes_of(
            &transform_texture_coords_matrix.transpose(),
        ));
        result[128..132].copy_from_slice(&from_u8_color(background_color.0));
        result[132..136].copy_from_slice(&from_u8_color(background_color.1));
        result[136..140].copy_from_slice(&from_u8_color(background_color.2));
        result[140..144].copy_from_slice(&from_u8_color(background_color.3));

        result[144..148].copy_from_slice(&is_texture.to_le_bytes());
        // 12 bytes padding

        result
    }
}

const ARRAY_SIZE: usize = 100;
const TEXTURE_PARAMS_SIZE: usize = 80;
const COLOR_PARAMS_SIZE: usize = 80;
const BOX_SHADOW_PARAMS_SIZE: usize = 64;
const PARENT_BORDER_RADIUS_SIZE: usize = 32;


struct LayoutInfo {
    layout_type: u32,
    index: u32,
}

struct LayoutParamsShaderContent {
    layout_infos: Vec<LayoutInfo>,
    texture_params_content: bytes::Bytes,
    color_params_content: bytes::Bytes,
    box_shadow_params_content: bytes::Bytes,
    parent_border_radiuses: bytes::Bytes,
}

impl LayoutParamsShaderContent {
    fn new(render_layouts: Vec<RenderLayout>) -> Self {
        let mut layout_infos = Vec::new();
        let mut texture_params_content = Vec::new();
        let mut color_params_content = Vec::new();
        let mut box_shadow_params_content = Vec::new();

        for layout in render_layouts.iter() {
            let RenderLayout {
                top,
                left,
                width,
                height,
                rotation_degrees,
                border_radius,
                parent_border_radiuses,
                content,
            } = layout;
            let mut border_radius_bytes: [u8; 16] = [0u8; 16];
            border_radius_bytes[0..4].copy_from_slice(&border_radius.top_left.to_le_bytes());
            border_radius_bytes[4..8].copy_from_slice(&border_radius.top_right.to_le_bytes());
            border_radius_bytes[8..12].copy_from_slice(&border_radius.bottom_right.to_le_bytes());
            border_radius_bytes[12..16].copy_from_slice(&border_radius.bottom_left.to_le_bytes());

            match content {
                RenderLayoutContent::ChildNode {
                    index: _,
                    crop,
                    border_color,
                    border_width,
                } => {
                    layout_infos.push(LayoutInfo {
                        layout_type: 0,
                        index: texture_params_content.len() as u32,
                    });
                    let mut texture_params = [0; TEXTURE_PARAMS_SIZE];
                    texture_params[0..16].copy_from_slice(&border_radius_bytes);
                    texture_params[16..32].copy_from_slice(&color_to_bytes(*border_color));
                    texture_params[32..36].copy_from_slice(&top.to_le_bytes());
                    texture_params[36..40].copy_from_slice(&left.to_le_bytes());
                    texture_params[40..44].copy_from_slice(&width.to_le_bytes());
                    texture_params[44..48].copy_from_slice(&height.to_le_bytes());
                    texture_params[48..52].copy_from_slice(&crop.top.to_le_bytes());
                    texture_params[52..56].copy_from_slice(&crop.left.to_le_bytes());
                    texture_params[56..60].copy_from_slice(&crop.width.to_le_bytes());
                    texture_params[60..64].copy_from_slice(&crop.height.to_le_bytes());
                    texture_params[64..68].copy_from_slice(&rotation_degrees.to_le_bytes());
                    texture_params[68..72].copy_from_slice(&border_width.to_le_bytes());
                    texture_params_content.push(texture_params);
                }
                RenderLayoutContent::Color {
                    color,
                    border_color,
                    border_width,
                } => {
                    layout_infos.push(LayoutInfo {
                        layout_type: 1,
                        index: color_params_content.len() as u32,
                    });
                    let mut color_params = [0; COLOR_PARAMS_SIZE];
                    color_params[0..16].copy_from_slice(&border_radius_bytes);
                    color_params[16..32].copy_from_slice(&color_to_bytes(*border_color));
                    color_params[32..48].copy_from_slice(&color_to_bytes(*color));
                    color_params[48..52].copy_from_slice(&top.to_le_bytes());
                    color_params[52..56].copy_from_slice(&left.to_le_bytes());
                    color_params[56..60].copy_from_slice(&width.to_le_bytes());
                    color_params[60..64].copy_from_slice(&height.to_le_bytes());
                    color_params[64..68].copy_from_slice(&rotation_degrees.to_le_bytes());
                    color_params[68..72].copy_from_slice(&border_width.to_le_bytes());

                    color_params_content.push(color_params);
                }
                RenderLayoutContent::BoxShadow { color, blur_radius } => {
                    layout_infos.push(LayoutInfo {
                        layout_type: 2,
                        index: box_shadow_params_content.len() as u32,
                    });
                    let mut box_shadow_params = [0; BOX_SHADOW_PARAMS_SIZE];
                    box_shadow_params[0..16].copy_from_slice(&border_radius_bytes);
                    box_shadow_params[16..32].copy_from_slice(&color_to_bytes(*color));
                    box_shadow_params[32..36].copy_from_slice(&top.to_le_bytes());
                    box_shadow_params[36..40].copy_from_slice(&left.to_le_bytes());
                    box_shadow_params[40..44].copy_from_slice(&width.to_le_bytes());
                    box_shadow_params[44..48].copy_from_slice(&height.to_le_bytes());
                    box_shadow_params[48..52].copy_from_slice(&rotation_degrees.to_le_bytes());
                    box_shadow_params[52..56].copy_from_slice(&blur_radius.to_le_bytes());
                    box_shadow_params_content.push(box_shadow_params);
                }
            }
        }

        let texture_params_content_bytes = (0..ARRAY_SIZE)
            .into_iter()
            .map(|i| match texture_params_content.get(i) {
                Some(params) => params,
                None => &[0; TEXTURE_PARAMS_SIZE],
            })
            .flatten()
            .copied()
            .collect::<Vec<u8>>();

        let color_params_content_bytes = (0..ARRAY_SIZE)
            .into_iter()
            .map(|i| match color_params_content.get(i) {
                Some(params) => params,
                None => &[0; COLOR_PARAMS_SIZE],
            })
            .flatten()
            .copied()
            .collect::<Vec<u8>>();

        let box_shadow_params_content_bytes = (0..ARRAY_SIZE)
            .into_iter()
            .map(|i| match box_shadow_params_content.get(i) {
                Some(params) => params,
                None => &[0; BOX_SHADOW_PARAMS_SIZE],
            })
            .flatten()
            .copied()
            .collect::<Vec<u8>>();

        Self {
            layout_infos,
            texture_params_content: texture_params_content_bytes.into(),
            color_params_content: color_params_content_bytes.into(),
            box_shadow_params_content: box_shadow_params_content_bytes.into(),
        }
    }
}

fn color_to_bytes(color: RGBAColor) -> [u8; 16] {
    let RGBAColor(r, g, b, a) = color;
    fn from_u8_color(value: u8) -> [u8; 4] {
        (value as f32 / 255.0).to_le_bytes()
    }
    let mut result = [0; 16];
    result[0..4].copy_from_slice(&from_u8_color(r));
    result[4..8].copy_from_slice(&from_u8_color(g));
    result[8..12].copy_from_slice(&from_u8_color(b));
    result[12..16].copy_from_slice(&from_u8_color(a));
    result
}
