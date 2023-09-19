use crate::renderer::{
    common_pipeline::{surface::SingleSurface, Sampler, Vertex, PRIMITIVE_STATE},
    texture::{BGRATexture, RGBATexture},
    WgpuCtx,
};

pub struct BGRAToRGBAConverter {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    surface: SingleSurface,
}

impl BGRAToRGBAConverter {
    pub fn new(device: &wgpu::Device, bgra_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("bgra_to_rgba.wgsl"));
        let sampler = Sampler::new(device);
        let surface = SingleSurface::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("BGRA to RGBA color converter render pipeline layout"),
            bind_group_layouts: &[bgra_bind_group_layout, &sampler.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("BGRA to RGBA color converter render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: PRIMITIVE_STATE,
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,
            sampler,
            surface,
        }
    }

    pub fn convert(
        &self,
        ctx: &WgpuCtx,
        src: (&BGRATexture, &wgpu::BindGroup),
        dst: &RGBATexture,
        clear_dst: bool,
    ) {
        let load = match clear_dst {
            true => wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            false => wgpu::LoadOp::Load,
        };

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("BGRA to RGBA color converter command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("BGRA to RGBA color converter render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst.texture().view,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: true },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, src.1, &[]);
            render_pass.set_bind_group(1, &self.sampler.bind_group, &[]);

            self.surface.draw(&mut render_pass);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }
}
