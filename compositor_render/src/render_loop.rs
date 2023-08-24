use std::{collections::HashMap, time::Duration};

use compositor_common::{
    scene::{InputId, NodeId, OutputId},
    util::RGBColor,
    Frame,
};
use log::error;

use crate::{
    frame_set::FrameSet,
    renderer::{
        scene::{Node, Scene, SceneError},
        RenderCtx,
    },
};

pub(super) fn populate_inputs(
    ctx: &RenderCtx,
    scene: &mut Scene,
    frame_set: &mut FrameSet<InputId>,
) -> Result<(), SceneError> {
    for (input_id, input_textures) in &mut scene.inputs {
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

    for (input_id, input_textures) in &mut scene.inputs {
        let node = scene.nodes.node(&input_id.0)?;
        if let Some(input_textures) = input_textures.state() {
            let node_texture = node
                .output
                .ensure_size(ctx.wgpu_ctx, input_textures.resolution());
            ctx.wgpu_ctx.format.convert_yuv_to_rgba(
                ctx.wgpu_ctx,
                (input_textures.yuv_textures(), input_textures.bind_group()),
                node_texture.rgba_texture(),
            );
        } else {
            node.output.clear()
        }
    }
    Ok(())
}

pub(super) fn read_outputs(
    ctx: &RenderCtx,
    scene: &mut Scene,
    pts: Duration,
) -> Result<HashMap<OutputId, Frame>, SceneError> {
    let mut pending_downloads = Vec::with_capacity(scene.outputs.len());
    for (output_id, (node_id, output_texture)) in &scene.outputs {
        let node = scene.nodes.node(node_id)?;
        match node.output.state() {
            Some(node) => {
                ctx.wgpu_ctx.format.convert_rgba_to_yuv(
                    ctx.wgpu_ctx,
                    ((node.rgba_texture()), (node.bind_group())),
                    output_texture.yuv_textures(),
                );
                let yuv_pending = output_texture.start_download(ctx.wgpu_ctx);
                pending_downloads.push((
                    output_id,
                    yuv_pending,
                    output_texture.resolution().to_owned(),
                ));
            }
            None => {
                let (y, u, v) = RGBColor::BLACK.to_yuv();
                ctx.wgpu_ctx.utils.fill_r8_with_value(
                    ctx.wgpu_ctx,
                    output_texture.yuv_textures().plane(0),
                    y,
                );
                ctx.wgpu_ctx.utils.fill_r8_with_value(
                    ctx.wgpu_ctx,
                    output_texture.yuv_textures().plane(1),
                    u,
                );
                ctx.wgpu_ctx.utils.fill_r8_with_value(
                    ctx.wgpu_ctx,
                    output_texture.yuv_textures().plane(2),
                    v,
                );
            }
        };
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
    Ok(result)
}

pub(super) fn run_transforms(
    ctx: &mut RenderCtx,
    scene: &mut Scene,
    pts: Duration,
) -> Result<(), SceneError> {
    let node_id_iter = render_order_iter(scene)?;
    for node_id in node_id_iter {
        let NodeRenderPass { node, inputs } = scene.nodes.node_render_pass(&node_id)?;
        let input_textures: Vec<_> = inputs
            .iter()
            .map(|node| (&node.node_id, &node.output))
            .collect();
        node.renderer
            .render(ctx, &input_textures, &node.output, pts)
    }
    Ok(())
}

pub struct NodeRenderPass<'a> {
    pub node: &'a mut Node,
    pub inputs: Vec<&'a Node>,
}

/// Returns iterator that guarantees that nodes will be visited in
/// the order that ensures that inputs of the current node were
/// already visited in previous iterations
fn render_order_iter(scene: &Scene) -> Result<impl Iterator<Item = NodeId>, SceneError> {
    let mut queue = vec![];
    for (node_id, _) in scene.outputs.values() {
        enqueue_nodes(node_id, &mut queue, scene)?;
    }
    Ok(queue.into_iter())
}

fn enqueue_nodes(
    parent_node_id: &NodeId,
    queue: &mut Vec<NodeId>,
    scene: &Scene,
) -> Result<(), SceneError> {
    let already_in_queue = queue.iter().any(|i| i == parent_node_id);
    if already_in_queue {
        return Ok(());
    }
    let parent_node = scene.nodes.node(parent_node_id)?;
    for child in &parent_node.inputs {
        enqueue_nodes(child, queue, scene)?;
    }
    queue.push(parent_node_id.clone());
    Ok(())
}
