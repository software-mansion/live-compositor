use std::sync::{Arc, OnceLock};

use tracing::error;

use crate::wgpu::{
    common_pipeline::{self, CreateShaderError, Sampler},
    texture::{NodeTexture, NodeTextureState},
    WgpuCtx, WgpuErrorScope,
};

use super::params::LayoutBindGroups;

static LAYOUT_SHADER_BIND_GROUP_2_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub fn bind_group_2_layout(wgpu_ctx: &WgpuCtx) -> &wgpu::BindGroupLayout {
    LAYOUT_SHADER_BIND_GROUP_2_LAYOUT.get_or_init(|| {
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
            })
    })
}

#[derive(Debug)]
pub struct LayoutShader {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
}

impl LayoutShader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        let scope = WgpuErrorScope::push(&wgpu_ctx.device);

        let shader_module = wgpu_ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("./apply_layouts.wgsl"));
        let result = Self::new_pipeline(wgpu_ctx, shader_module)?;

        scope.pop(&wgpu_ctx.device)?;

        Ok(result)
    }

    fn new_pipeline(
        wgpu_ctx: &Arc<WgpuCtx>,
        shader_module: wgpu::ShaderModule,
    ) -> Result<Self, CreateShaderError> {
        let sampler = Sampler::new(&wgpu_ctx.device);

        let texture_bgl = common_pipeline::single_texture_bind_group_layout(&wgpu_ctx.device);
        let bind_group_1_layout = &wgpu_ctx.uniform_bgl;
        let bind_group_2_layout = bind_group_2_layout(wgpu_ctx);

        let pipeline_layout =
            wgpu_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shader transformation pipeline layout"),
                    bind_group_layouts: &[
                        texture_bgl,
                        bind_group_1_layout,
                        bind_group_2_layout,
                        &sampler.bind_group_layout,
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..4,
                    }],
                });

        let pipeline = common_pipeline::create_render_pipeline(
            &wgpu_ctx.device,
            &pipeline_layout,
            &shader_module,
        );

        Ok(Self { pipeline, sampler })
    }

    pub fn render(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        param_bind_groups: LayoutBindGroups,
        textures: &[Option<&NodeTexture>],
        target: &NodeTextureState,
    ) {
        let LayoutBindGroups {
            bind_group_1,
            bind_groups_2,
        } = param_bind_groups;
        let input_texture_bgs: Vec<wgpu::BindGroup> = self.input_textures_bg(wgpu_ctx, textures);

        let mut encoder = wgpu_ctx.device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &target.rgba_texture().texture().view,
                    resolve_target: None,
                })],
                // TODO: depth stencil attachments
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if input_texture_bgs.len() != bind_groups_2.len() {
                error!(
                    "Input textures bind groups count ({}) doesn't match params bind groups count ({})",
                    input_texture_bgs.len(),
                    bind_groups_2.len()
                );
            }

            for (texture_bg, bind_group_2) in input_texture_bgs.iter().zip(bind_groups_2.iter()) {
                render_pass.set_pipeline(&self.pipeline);

                render_pass.set_bind_group(0, texture_bg, &[]);
                render_pass.set_bind_group(1, &bind_group_1, &[]);
                render_pass.set_bind_group(2, bind_group_2, &[]);
                render_pass.set_bind_group(3, &self.sampler.bind_group, &[]);

                wgpu_ctx.plane.draw(&mut render_pass);
            }
        }
        wgpu_ctx.queue.submit(Some(encoder.finish()));
    }

    fn input_textures_bg(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        textures: &[Option<&NodeTexture>],
    ) -> Vec<wgpu::BindGroup> {
        textures
            .iter()
            .map(|texture| {
                texture
                    .and_then(|texture| texture.state())
                    .map(|state| &state.rgba_texture().texture().view)
                    .unwrap_or(&wgpu_ctx.empty_texture.view)
            })
            .map(|view| {
                wgpu_ctx
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: common_pipeline::single_texture_bind_group_layout(&wgpu_ctx.device),
                        label: None,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(view),
                        }],
                    })
            })
            .collect()
    }
}
