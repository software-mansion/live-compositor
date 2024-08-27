use std::{collections::HashMap, time::Duration};

use tracing::error;

use crate::{
    scene::RGBColor,
    state::{node::RenderNode, render_graph::RenderGraph, RenderCtx},
    wgpu::texture::{InputTexture, NodeTexture, PlanarYuvPendingDownload},
    Frame, FrameData, FrameSet, InputId, OutputFrameFormat, OutputId, Resolution,
};

pub(super) fn populate_inputs(
    ctx: &RenderCtx,
    scene: &mut RenderGraph,
    mut frame_set: FrameSet<InputId>,
) {
    for (input_id, (_node_texture, input_textures)) in &mut scene.inputs {
        let Some(frame) = frame_set.frames.remove(input_id) else {
            input_textures.clear();
            continue;
        };
        if Duration::saturating_sub(frame_set.pts, ctx.stream_fallback_timeout) > frame.pts {
            input_textures.clear();
            continue;
        }

        input_textures.upload(ctx.wgpu_ctx, frame);
    }

    ctx.wgpu_ctx.queue.submit([]);

    for (node_texture, input_textures) in scene.inputs.values_mut() {
        input_textures.convert_to_node_texture(ctx.wgpu_ctx, node_texture);
    }
}

enum PartialOutputFrame<'a, F>
where
    F: FnOnce() -> Result<bytes::Bytes, wgpu::BufferAsyncError> + 'a,
{
    PendingYuvDownload {
        output_id: OutputId,
        pending_download: PlanarYuvPendingDownload<'a, F, wgpu::BufferAsyncError>,
        resolution: Resolution,
    },
    CompleteFrame {
        output_id: OutputId,
        frame: Frame,
    },
}

pub(super) fn read_outputs(
    ctx: &RenderCtx,
    scene: &mut RenderGraph,
    pts: Duration,
) -> HashMap<OutputId, Frame> {
    let mut partial_textures = Vec::with_capacity(scene.outputs.len());
    for (output_id, output) in &scene.outputs {
        match output.root.output_texture(&scene.inputs).state() {
            Some(node) => match output.output_format {
                OutputFrameFormat::PlanarYuv420Bytes => {
                    ctx.wgpu_ctx.format.convert_rgba_to_yuv(
                        ctx.wgpu_ctx,
                        (node.rgba_texture(), node.bind_group()),
                        output.output_texture.yuv_textures(),
                    );
                    let pending_download = output.output_texture.start_download(ctx.wgpu_ctx);
                    partial_textures.push(PartialOutputFrame::PendingYuvDownload {
                        output_id: output_id.clone(),
                        pending_download,
                        resolution: output.output_texture.resolution().to_owned(),
                    });
                }
                OutputFrameFormat::RgbaWgpuTexture => {
                    let texture = node
                        .rgba_texture()
                        .texture()
                        .copy_wgpu_texture(ctx.wgpu_ctx);
                    let size = texture.size();
                    let frame = Frame {
                        data: FrameData::Rgba8UnormWgpuTexture(texture.into()),
                        resolution: Resolution {
                            width: size.width as usize,
                            height: size.height as usize,
                        },
                        pts,
                    };
                    partial_textures.push(PartialOutputFrame::CompleteFrame {
                        output_id: output_id.clone(),
                        frame,
                    })
                }
            },
            None => match output.output_format {
                OutputFrameFormat::PlanarYuv420Bytes => {
                    let (y, u, v) = RGBColor::BLACK.to_yuv();
                    ctx.wgpu_ctx.utils.fill_r8_with_value(
                        ctx.wgpu_ctx,
                        output.output_texture.yuv_textures().plane(0),
                        y,
                    );
                    ctx.wgpu_ctx.utils.fill_r8_with_value(
                        ctx.wgpu_ctx,
                        output.output_texture.yuv_textures().plane(1),
                        u,
                    );
                    ctx.wgpu_ctx.utils.fill_r8_with_value(
                        ctx.wgpu_ctx,
                        output.output_texture.yuv_textures().plane(2),
                        v,
                    );

                    let pending_download = output.output_texture.start_download(ctx.wgpu_ctx);
                    partial_textures.push(PartialOutputFrame::PendingYuvDownload {
                        output_id: output_id.clone(),
                        pending_download,
                        resolution: output.output_texture.resolution().to_owned(),
                    });
                }
                OutputFrameFormat::RgbaWgpuTexture => {
                    let output_resolution = output.output_texture.resolution();
                    let wgpu_texture =
                        ctx.wgpu_ctx
                            .device
                            .create_texture(&wgpu::TextureDescriptor {
                                label: None,
                                size: wgpu::Extent3d {
                                    width: output_resolution.width as u32,
                                    height: output_resolution.height as u32,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                usage: wgpu::TextureUsages::COPY_SRC
                                    | wgpu::TextureUsages::COPY_DST
                                    | wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                            });
                    let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let compositor_texture = crate::wgpu::texture::Texture {
                        texture: wgpu_texture,
                        view,
                    };
                    ctx.wgpu_ctx
                        .utils
                        .fill_r8_with_value(ctx.wgpu_ctx, &compositor_texture, 0.0);

                    let frame = Frame {
                        data: FrameData::Rgba8UnormWgpuTexture(compositor_texture.texture.into()),
                        resolution: output_resolution,
                        pts,
                    };
                    partial_textures.push(PartialOutputFrame::CompleteFrame {
                        output_id: output_id.clone(),
                        frame,
                    })
                }
            },
        };
    }

    ctx.wgpu_ctx.device.poll(wgpu::MaintainBase::Wait);

    let mut result = HashMap::new();
    for partial in partial_textures {
        match partial {
            PartialOutputFrame::PendingYuvDownload {
                output_id,
                pending_download,
                resolution,
            } => {
                let data = match pending_download.wait() {
                    Ok(data) => data,
                    Err(err) => {
                        error!("Failed to download frame: {}", err);
                        continue;
                    }
                };
                let frame = Frame {
                    data,
                    resolution,
                    pts,
                };
                result.insert(output_id.clone(), frame);
            }

            PartialOutputFrame::CompleteFrame { output_id, frame } => {
                result.insert(output_id, frame);
            }
        }
    }
    result
}

pub(super) fn run_transforms(ctx: &mut RenderCtx, scene: &mut RenderGraph, pts: Duration) {
    for output in scene.outputs.values_mut() {
        render_node(ctx, &scene.inputs, pts, &mut output.root);
    }
}

pub(super) fn render_node(
    ctx: &mut RenderCtx,
    inputs: &HashMap<InputId, (NodeTexture, InputTexture)>,
    pts: Duration,
    node: &mut RenderNode,
) {
    for child_node in node.children.iter_mut() {
        render_node(ctx, inputs, pts, child_node);
    }

    let input_textures: Vec<_> = node
        .children
        .iter()
        .map(|node| node.output_texture(inputs))
        .collect();
    node.renderer
        .render(ctx, &input_textures, &mut node.output, pts);
}
