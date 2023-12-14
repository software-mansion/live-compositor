use std::{num::NonZeroU32, sync::Arc};

use wgpu::ShaderStages;

use crate::wgpu::{
    common_pipeline::{Sampler, Vertex},
    shader::{CreateShaderError, FRAGMENT_ENTRYPOINT_NAME, VERTEX_ENTRYPOINT_NAME},
    texture::{NodeTextureState, Texture},
    WgpuCtx, WgpuErrorScope,
};

#[derive(Debug)]
pub(super) struct WebRendererShader {
    wgpu_ctx: Arc<WgpuCtx>,
    pipeline: wgpu::RenderPipeline,
    textures_bgl: wgpu::BindGroupLayout,
    sampler: Sampler,
}

impl WebRendererShader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        let scope = WgpuErrorScope::push(&wgpu_ctx.device);

        let shader_module = wgpu_ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../web_renderer/render_website.wgsl"));
        let sampler = Sampler::new(&wgpu_ctx.device);
        let textures_bgl =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("web renderer textures bgl"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: NonZeroU32::new(super::MAX_WEB_RENDERER_TEXTURES_COUNT),
                    }],
                });

        let pipeline_layout =
            wgpu_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Web renderer pipeline layout"),
                    bind_group_layouts: &[
                        &textures_bgl,
                        &wgpu_ctx.shader_parameters_bind_group_layout,
                        &sampler.bind_group_layout,
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..4,
                    }],
                });

        let pipeline = wgpu_ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shader transformation pipeline :^)"),
                depth_stencil: None,
                primitive: wgpu::PrimitiveState {
                    conservative: false,
                    cull_mode: Some(wgpu::Face::Back),
                    front_face: wgpu::FrontFace::Ccw,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    unclipped_depth: false,
                },
                vertex: wgpu::VertexState {
                    buffers: &[Vertex::LAYOUT],
                    module: &shader_module,
                    entry_point: VERTEX_ENTRYPOINT_NAME,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: FRAGMENT_ENTRYPOINT_NAME,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        write_mask: wgpu::ColorWrites::all(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    })],
                }),
                layout: Some(&pipeline_layout),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        scope.pop(&wgpu_ctx.device)?;

        Ok(Self {
            wgpu_ctx: wgpu_ctx.clone(),
            pipeline,
            textures_bgl,
            sampler,
        })
    }

    pub(super) fn render(
        &self,
        params: &wgpu::BindGroup,
        textures: &[Option<&Texture>],
        target: &NodeTextureState,
    ) {
        let mut texture_views: Vec<&wgpu::TextureView> = textures
            .iter()
            .map(|texture| match texture {
                Some(texture) => &texture.view,
                None => &self.wgpu_ctx.empty_texture.view,
            })
            .collect();

        texture_views.extend(
            (texture_views.len()..super::MAX_WEB_RENDERER_TEXTURES_COUNT as usize)
                .map(|_| &self.wgpu_ctx.empty_texture.view),
        );

        let input_textures_bg =
            self.wgpu_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Web renderer input textures bgl"),
                    layout: &self.textures_bgl,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&texture_views),
                    }],
                });

        let mut encoder = self
            .wgpu_ctx
            .device
            .create_command_encoder(&Default::default());

        let mut render_plane = |input_id: i32, clear: bool| {
            let load = match clear {
                true => wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                false => wgpu::LoadOp::Load,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations { load, store: true },
                    view: &target.rgba_texture().texture().view,
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_push_constants(
                ShaderStages::VERTEX_FRAGMENT,
                0,
                &(textures.len() as u32).to_le_bytes(),
            );

            render_pass.set_bind_group(0, &input_textures_bg, &[]);
            render_pass.set_bind_group(1, params, &[]);
            render_pass.set_bind_group(2, &self.sampler.bind_group, &[]);

            self.wgpu_ctx
                .plane_cache
                .plane(input_id)
                .unwrap()
                .draw(&mut render_pass);
        };

        render_plane(0, false);
        for input_id in 1..textures.len() {
            render_plane(input_id as i32, true);
        }

        self.wgpu_ctx.queue.submit(Some(encoder.finish()));
    }
}
