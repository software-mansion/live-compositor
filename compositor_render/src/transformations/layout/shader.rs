use std::sync::Arc;

use tracing::{error, info};

use crate::{
    scene::RGBAColor,
    transformations::layout::{BorderRadius, Crop, RenderLayoutContent},
    wgpu::{
        common_pipeline::{self, CreateShaderError, Sampler},
        texture::{NodeTexture, NodeTextureState},
        WgpuCtx, WgpuErrorScope,
    },
    Resolution,
};

use super::{params::ParamsBindGroups, vertices_transformation_matrix, RenderLayout};

#[derive(Debug)]
pub struct LayoutShader {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    texture_bgl: wgpu::BindGroupLayout,
    params_bind_groups: ParamsBindGroups,
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
        let texture_bgl = common_pipeline::create_single_texture_bgl(&wgpu_ctx.device);
        let params_bind_groups = ParamsBindGroups::new(wgpu_ctx);

        let pipeline_layout =
            wgpu_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shader transformation pipeline layout"),
                    bind_group_layouts: &[
                        &texture_bgl,
                        &params_bind_groups.bind_group_1_layout,
                        &params_bind_groups.bind_group_2_layout,
                        &sampler.bind_group_layout,
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..16,
                    }],
                });

        let pipeline = common_pipeline::create_render_pipeline(
            &wgpu_ctx.device,
            &pipeline_layout,
            &shader_module,
        );

        Ok(Self {
            pipeline,
            sampler,
            texture_bgl,
            params_bind_groups,
        })
    }

    pub fn render(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        output_resolution: Resolution,
        layouts: Vec<RenderLayout>,
        textures: &mut Vec<Option<&NodeTexture>>,
        target: &NodeTextureState,
    ) {
        // let new_layouts = vec![
        //     RenderLayout {
        //         top: 0.0,
        //         left: 0.0,
        //         width: 960.0,
        //         height: 540.0,
        //         rotation_degrees: 0.0,
        //         border_radius: super::BorderRadius {
        //             top_left: 50.0,
        //             top_right: 100.0,
        //             bottom_right: 150.0,
        //             bottom_left: 200.0,
        //         },
        //         parent_border_radiuses: vec![],
        //         content: RenderLayoutContent::ChildNode {
        //             index: 0,
        //             crop: Crop {
        //                 top: 0.0,
        //                 left: 0.0,
        //                 width: 720.0,
        //                 height: 480.0,
        //             },
        //             border_color: RGBAColor(255, 0, 0, 255),
        //             border_width: 20.0,
        //         },
        //     },
        //     RenderLayout {
        //         top: 540.0,
        //         left: 960.0,
        //         width: 960.0,
        //         height: 540.0,
        //         rotation_degrees: 0.0,
        //         border_radius: BorderRadius {
        //             top_left: 0.0,
        //             top_right: 100.0,
        //             bottom_right: 200.0,
        //             bottom_left: 300.0,
        //         },
        //         parent_border_radiuses: vec![],
        //         content: RenderLayoutContent::Color {
        //             color: RGBAColor(0, 255, 0, 255),
        //             border_color: RGBAColor(0, 0, 0, 0),
        //             border_width: 0.0,
        //         },
        //     },
        // ];

        let layout_infos = self
            .params_bind_groups
            .update(wgpu_ctx, output_resolution, layouts);
        // textures.push(None);
        let input_texture_bgs: Vec<wgpu::BindGroup> = self.input_textures_bg(wgpu_ctx, &textures);

        if layout_infos.len() != input_texture_bgs.len() {
            error!(
                "Layout infos len ({:?}) and textures bind groups count ({:?}) mismatch",
                layout_infos.len(),
                input_texture_bgs.len()
            );
        }

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

            for (index, (texture_bg, layout_info)) in input_texture_bgs
                .iter()
                .zip(layout_infos.iter())
                .take(100)
                .enumerate()
            {
                render_pass.set_pipeline(&self.pipeline);

                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    0,
                    &layout_info.to_bytes(),
                );

                render_pass.set_bind_group(0, texture_bg, &[]);
                render_pass.set_bind_group(1, &self.params_bind_groups.bind_group_1, &[]);
                render_pass.set_bind_group(2, &self.params_bind_groups.bind_groups_2[index].0, &[]);
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
                        layout: &self.texture_bgl,
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
