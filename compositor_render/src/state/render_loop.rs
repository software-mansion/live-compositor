use std::{collections::HashMap, time::Duration};

use log::error;

use crate::{
    scene::RGBColor,
    state::{node::RenderNode, render_graph::RenderGraph, RenderCtx},
    wgpu::texture::{InputTexture, NodeTexture},
    Frame, FrameSet, InputId, OutputId,
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

pub(super) fn read_outputs(
    ctx: &RenderCtx,
    scene: &mut RenderGraph,
    pts: Duration,
) -> HashMap<OutputId, Frame> {
    let mut pending_downloads = Vec::with_capacity(scene.outputs.len());
    for (output_id, output) in &scene.outputs {
        match output.root.output_texture(&scene.inputs).state() {
            Some(node) => {
                ctx.wgpu_ctx.format.convert_rgba_to_yuv(
                    ctx.wgpu_ctx,
                    (node.rgba_texture(), node.bind_group()),
                    output.output_texture.yuv_textures(),
                );
            }
            None => {
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
            }
        };
        let yuv_pending = output.output_texture.start_download(ctx.wgpu_ctx);
        pending_downloads.push((
            output_id,
            yuv_pending,
            output.output_texture.resolution().to_owned(),
        ));
    }
    ctx.wgpu_ctx.device.poll(wgpu::MaintainBase::Wait);

    let mut result = HashMap::new();
    for (output_id, yuv_pending, resolution) in pending_downloads {
        let yuv_data = match yuv_pending.wait() {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to download frame: {}", err);
                continue;
            }
        };
        result.insert(
            output_id.clone(),
            Frame {
                data: yuv_data,
                resolution,
                pts,
            },
        );
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
