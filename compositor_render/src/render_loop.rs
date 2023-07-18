use std::{collections::HashMap, sync::Arc};

use compositor_common::{scene::InputId, Frame};
use log::error;

use crate::renderer::{
    scene::{Node, Scene},
    texture::Texture,
    RenderCtx,
};

pub fn render_scene(ctx: &RenderCtx, scene: &Scene, mut frames: HashMap<InputId, Arc<Frame>>) {
    for node in graph_iter(scene) {
        match *node {
            Node::Input {
                ref node_id,
                ref yuv_textures,
                ..
            } => {
                Texture::upload_frame_to_textures(
                    &ctx.wgpu_ctx,
                    &yuv_textures,
                    frames.remove(&node_id).unwrap(),
                );
                error!("TODO: convert yuv to rgba")
            }
            Node::Transformation {
                ref output,
                ref node,
                ref inputs,
                ..
            } => {
                let sources = inputs
                    .iter()
                    .map(|node| (node.id(), node.output()))
                    .collect();
                node.render(ctx, &sources, &output)
            }
        }
    }
}

/// Returns iterator that guarantees that nodes will be visited in
/// the order that ensures that inputs of the current node were
/// already visited in previous iterations
fn graph_iter(scene: &Scene) -> impl Iterator<Item = Arc<Node>> {
    let mut queue = vec![];
    for (_id, (output_node, _output_texture)) in &scene.outputs {
        enqueue_nodes(&output_node, &mut queue);
    }
    queue.into_iter()
}

fn enqueue_nodes(parent_node: &Arc<Node>, queue: &mut Vec<Arc<Node>>) {
    let already_in_queue = queue
        .iter()
        .find(|i| Arc::ptr_eq(&parent_node, i))
        .is_some();
    if already_in_queue {
        return;
    }
    for child in parent_node.inputs().iter() {
        enqueue_nodes(&child, queue);
    }
    queue.push(parent_node.clone());
}
