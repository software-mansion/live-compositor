use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::{
    scene::{InputId, OutputId},
    Frame,
};

use crate::renderer::{
    scene::{Node, Scene},
    texture::Texture,
    RenderCtx,
};

pub(super) fn populate_inputs(
    ctx: &RenderCtx,
    scene: &Scene,
    frames: &mut HashMap<InputId, Arc<Frame>>,
) {
    for (input_id, (node, input_textures)) in &scene.inputs {
        Texture::upload_frame_to_textures(
            ctx.wgpu_ctx,
            &input_textures.textures.0,
            frames.remove(input_id).unwrap(),
        );
        ctx.wgpu_ctx.yuv_to_rgba_converter.convert(
            ctx.wgpu_ctx,
            (&input_textures.textures, &input_textures.bind_group),
            &node.output.texture,
        );
    }
}

pub(super) fn read_outputs(
    ctx: &RenderCtx,
    scene: &Scene,
    pts: Duration,
) -> HashMap<OutputId, Arc<Frame>> {
    let mut result = HashMap::new();
    for (node_id, (node, output)) in &scene.outputs {
        ctx.wgpu_ctx.rgba_to_yuv_converter.convert(
            ctx.wgpu_ctx,
            (&node.output.texture, &node.output.bind_group),
            &output.yuv_textures,
        );
        let yuv_data = output.download(ctx.wgpu_ctx);
        result.insert(
            node_id.clone(),
            Arc::new(Frame {
                data: yuv_data,
                resolution: output.resolution,
                pts,
            }),
        );
    }
    result
}

pub(super) fn run_transforms(ctx: &RenderCtx, scene: &Scene) {
    for node in render_order_iter(scene) {
        let sources: Vec<_> = node
            .inputs
            .iter()
            .map(|node| (&node.node_id, &node.output))
            .collect();
        node.transform.render(ctx, &sources, &node.output)
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
