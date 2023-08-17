use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::{
    scene::{InputId, OutputId},
    Frame,
};
use log::error;

use crate::renderer::{
    scene::{Node, Scene},
    RenderCtx,
};

pub(super) fn populate_inputs(
    ctx: &RenderCtx,
    scene: &mut Scene,
    frames: &mut HashMap<InputId, Frame>,
) {
    for (input_id, (node, input_textures)) in &mut scene.inputs {
        let Some(frame) = frames.remove(input_id) else {
            continue;
        };
        node.output.ensure_size(ctx.wgpu_ctx, frame.resolution);
        input_textures.upload(ctx.wgpu_ctx, frame);
    }

    ctx.wgpu_ctx.queue.submit([]);

    for (node, input_textures) in scene.inputs.values() {
        ctx.wgpu_ctx.yuv_to_rgba_converter.convert(
            ctx.wgpu_ctx,
            (input_textures.yuv_textures(), input_textures.bind_group()),
            &node.output.rgba_texture(),
        );
    }
}

pub(super) fn read_outputs(
    ctx: &RenderCtx,
    scene: &Scene,
    pts: Duration,
) -> HashMap<OutputId, Frame> {
    let mut pending_downloads = Vec::with_capacity(scene.outputs.len());
    for (node_id, (node, output)) in &scene.outputs {
        ctx.wgpu_ctx.rgba_to_yuv_converter.convert(
            ctx.wgpu_ctx,
            (&node.output.rgba_texture(), &node.output.bind_group()),
            output.yuv_textures(),
        );
        let yuv_pending = output.start_download(ctx.wgpu_ctx);
        pending_downloads.push((node_id, yuv_pending, output.resolution().to_owned()));
    }
    ctx.wgpu_ctx.device.poll(wgpu::MaintainBase::Wait);

    let mut result = HashMap::new();
    for (node_id, yuv_pending, resolution) in pending_downloads {
        let yuv_data = match yuv_pending.wait() {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to download frame: {}", err);
                continue;
            }
        };
        result.insert(
            node_id.clone(),
            Frame {
                data: yuv_data,
                resolution,
                pts,
            },
        );
    }
    result
}

pub(super) fn run_transforms(ctx: &mut RenderCtx, scene: &Scene, pts: Duration) {
    for node in render_order_iter(scene) {
        let sources: Vec<_> = node
            .inputs
            .iter()
            .map(|node| (&node.node_id, &node.output))
            .collect();
        node.transform.render(ctx, &sources, &node.output, pts)
    }
}

/// Returns iterator that guarantees that nodes will be visited in
/// the order that ensures that inputs of the current node were
/// already visited in previous iterations
fn render_order_iter(scene: &Scene) -> impl Iterator<Item = Arc<Node>> {
    let mut queue = vec![];
    for (output_node, _output_texture) in scene.outputs.values() {
        enqueue_nodes(output_node, &mut queue);
    }
    queue.into_iter()
}

fn enqueue_nodes(parent_node: &Arc<Node>, queue: &mut Vec<Arc<Node>>) {
    let already_in_queue = queue.iter().any(|i| Arc::ptr_eq(parent_node, i));
    if already_in_queue {
        return;
    }
    for child in parent_node.inputs.iter() {
        enqueue_nodes(child, queue);
    }
    queue.push(parent_node.clone());
}
