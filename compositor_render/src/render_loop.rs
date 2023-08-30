use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use compositor_common::{
    scene::{InputId, NodeId, OutputId},
    util::RGBColor,
    Frame,
};
use log::error;

use crate::{
    frame_set::FrameSet,
    renderer::{
        scene::{Node, Scene, SceneError, SceneNodesSet},
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
        let node = scene.nodes.node_mut(&input_id.0)?;
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
        let node = scene.nodes.node_or_fallback(node_id)?;
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
    let mut already_rendered = HashSet::new();
    for (node_id, _) in scene.outputs.values() {
        render_node(ctx, &mut scene.nodes, pts, node_id, &mut already_rendered)?;
    }
    Ok(())
}

pub(super) fn render_node(
    ctx: &mut RenderCtx,
    nodes: &mut SceneNodesSet,
    pts: Duration,
    node_id: &NodeId,
    already_rendered: &mut HashSet<NodeId>,
) -> Result<(), SceneError> {
    if already_rendered.contains(node_id) {
        return Ok(());
    }
    // Make sure all input are rendered
    {
        let input_ids: Vec<_> = nodes.node(node_id)?.inputs.to_vec();
        for input_id in input_ids {
            render_node(ctx, nodes, pts, &input_id, already_rendered)?;
        }
    }
    // Try to render node
    //
    // - If node texture is empty after the render and fallback_id was
    // defined return that Some(fallback_id) from this block.
    // - If node texture is not empty return None, even if fallback_id
    // was defined
    let fallback_id = {
        let NodeRenderPass { node, inputs } = nodes.node_render_pass(node_id)?;
        let input_textures: Vec<_> = inputs
            .iter()
            .map(|(node_id, node)| (node_id, &node.output))
            .collect();
        node.renderer
            .render(ctx, &input_textures, &mut node.output, pts);

        match node.output.is_empty() {
            true => node.fallback.clone(),
            false => None,
        }
    };

    // Try to render a fallback
    if let Some(fallback_id) = fallback_id {
        render_node(ctx, nodes, pts, &fallback_id, already_rendered)?;
    }

    Ok(())
}

pub(crate) struct NodeRenderPass<'a> {
    pub node: &'a mut Node,
    /// NodeId identifies input pad, but Node might refer
    /// to a node with different id if fallback are in use
    pub inputs: Vec<(NodeId, &'a Node)>,
}
