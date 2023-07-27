use std::{collections::HashMap, sync::Arc};

use compositor_common::{scene::InputId, Frame};
use log::error;

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
    for (input_id, (_, yuv_textures)) in &scene.inputs {
        Texture::upload_frame_to_textures(
            ctx.wgpu_ctx,
            yuv_textures,
            frames.remove(input_id).unwrap(),
        );
        error!("TODO: convert yuv to rgba")
    }
}

pub(super) fn run_transforms(ctx: &RenderCtx, scene: &Scene) {
    for node in graph_iter(scene) {
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
fn graph_iter(scene: &Scene) -> impl Iterator<Item = Arc<Node>> {
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
