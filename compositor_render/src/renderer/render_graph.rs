use std::collections::HashMap;
use std::fmt::Display;

use compositor_common::scene::{InputId, OutputId};
use log::error;

use crate::scene::{self, OutputNode};
use crate::wgpu::texture::{InputTexture, OutputTexture};
use crate::{error::UpdateSceneError, wgpu::WgpuErrorScope};

use super::NodeRenderPass;
use super::{node::RenderNode, RenderCtx};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct RenderGraph {
    pub nodes: RenderNodesSet,
    pub outputs: HashMap<OutputId, (NodeId, OutputTexture)>,
    pub inputs: HashMap<InputId, (NodeId, InputTexture)>,
}

#[derive(Debug)]
struct NodeIdProvider(NodeId);

impl NodeIdProvider {
    fn new() -> Self {
        Self(NodeId(0))
    }

    fn next(&mut self) -> NodeId {
        self.0 = NodeId(self.0 .0 + 1);
        self.0
    }
}

impl RenderGraph {
    pub fn empty() -> Self {
        Self {
            nodes: RenderNodesSet::new(),
            outputs: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    pub(crate) fn update(
        &mut self,
        ctx: &RenderCtx,
        output_nodes: Vec<OutputNode>,
    ) -> Result<(), UpdateSceneError> {
        // TODO: If we want nodes to be stateful we could try reusing nodes instead
        //       of recreating them on every scene update
        let scope = WgpuErrorScope::push(&ctx.wgpu_ctx.device);

        let mut new_nodes = HashMap::new();
        let mut inputs = HashMap::new();
        let mut id_provider = NodeIdProvider::new();
        let outputs = output_nodes
            .into_iter()
            .map(|output| {
                let node_id = Self::ensure_node(
                    ctx,
                    output.node,
                    &mut inputs,
                    &mut new_nodes,
                    &mut id_provider,
                )?;
                let output_texture = OutputTexture::new(ctx.wgpu_ctx, output.resolution);
                Ok((output.output_id.clone(), (node_id, output_texture)))
            })
            .collect::<Result<_, UpdateSceneError>>()?;

        scope.pop(&ctx.wgpu_ctx.device)?;

        self.inputs = inputs;
        self.outputs = outputs;
        self.nodes = RenderNodesSet { nodes: new_nodes };

        Ok(())
    }

    fn ensure_node(
        ctx: &RenderCtx,
        node: scene::Node,
        inputs: &mut HashMap<InputId, (NodeId, InputTexture)>,
        new_nodes: &mut HashMap<NodeId, RenderNode>,
        id_provider: &mut NodeIdProvider,
    ) -> Result<NodeId, UpdateSceneError> {
        // check if input stream already registered
        if let scene::NodeKind::InputStream(input) = &node.kind {
            if let Some((node_id, _)) = inputs.get(&input.input_id) {
                return Ok(*node_id);
            }
        }

        let node_id = id_provider.next();
        let input_pads: Vec<NodeId> = node
            .children
            .into_iter()
            .map(|node| Self::ensure_node(ctx, node, inputs, new_nodes, id_provider))
            .collect::<Result<Vec<_>, _>>()?;

        match node.kind {
            scene::NodeKind::InputStream(input) => {
                let node = RenderNode::new_input();
                new_nodes.insert(node_id, node);
                inputs.insert(input.input_id.clone(), (node_id, InputTexture::new()));
            }
            scene::NodeKind::Shader(shader) => {
                let node = RenderNode::new_shader_node(ctx, input_pads, shader)
                    .map_err(|err| UpdateSceneError::CreateNodeError(err, node_id.0))?;
                new_nodes.insert(node_id, node);
            }
            scene::NodeKind::Layout(layout) => {
                let node = RenderNode::new_layout_node(ctx, input_pads, layout)
                    .map_err(|err| UpdateSceneError::CreateNodeError(err, node_id.0))?;
                new_nodes.insert(node_id, node);
            }
        }
        Ok(node_id)
    }
}

#[derive(Default)]
pub struct RenderNodesSet {
    nodes: HashMap<NodeId, RenderNode>,
}

impl RenderNodesSet {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn node(&self, node_id: &NodeId) -> Result<&RenderNode, InternalSceneError> {
        self.nodes
            .get(node_id)
            .ok_or(InternalSceneError::MissingNode(node_id.0))
    }

    pub fn node_mut(&mut self, node_id: &NodeId) -> Result<&mut RenderNode, InternalSceneError> {
        self.nodes
            .get_mut(node_id)
            .ok_or(InternalSceneError::MissingNode(node_id.0))
    }

    pub fn node_or_fallback<'a>(
        &'a self,
        node_id: &NodeId,
    ) -> Result<&'a RenderNode, InternalSceneError> {
        let nodes: HashMap<&NodeId, &RenderNode> = self.nodes.iter().collect();
        Self::find_fallback_node(&nodes, node_id)
    }

    /// Borrow all nodes that are needed to render node node_id.
    pub(crate) fn node_render_pass<'a>(
        &'a mut self,
        node_id: &NodeId,
    ) -> Result<NodeRenderPass<'a>, InternalSceneError> {
        let input_ids: Vec<NodeId> = self.node(node_id)?.inputs.to_vec();

        // Borrow all the references, Fallback technically can be applied on every
        // level, so the easiest approach is to just borrow everything
        let mut nodes_mut: HashMap<&NodeId, &mut RenderNode> = self.nodes.iter_mut().collect();

        // Extract mutable borrow for the node we will render.
        let node = nodes_mut
            .remove(&node_id)
            .ok_or(InternalSceneError::MissingNode(node_id.0))?;

        // Convert mutable borrows on rest of the nodes into immutable.
        // One input might be used multiple times, so we might need to
        // borrow it more than once, so it needs to be immutable.
        let nodes: HashMap<&NodeId, &RenderNode> = nodes_mut
            .into_iter()
            .map(|(id, node)| (id, &*node))
            .collect();

        // Get immutable borrows for inputs. For each input if node texture
        // is empty go through the fallback chain
        let inputs: Vec<(NodeId, &RenderNode)> = input_ids
            .into_iter()
            .map(|input_id| {
                let node = Self::find_fallback_node(&nodes, &input_id)?;
                // input_id and node.node_id are different if fallback is triggered
                Ok((input_id, node))
            })
            .collect::<Result<Vec<_>, InternalSceneError>>()?;
        Ok(NodeRenderPass { node, inputs })
    }

    fn find_fallback_node<'a>(
        nodes: &HashMap<&NodeId, &'a RenderNode>,
        node_id: &NodeId,
    ) -> Result<&'a RenderNode, InternalSceneError> {
        let mut node: &RenderNode = nodes
            .get(node_id)
            .ok_or(InternalSceneError::MissingNode(node_id.0))?;
        while node.output.is_empty() && node.fallback.is_some() {
            let fallback_id = node.fallback.unwrap();
            node = nodes
                .get(&fallback_id)
                .ok_or(InternalSceneError::MissingNode(fallback_id.0))?
        }
        Ok(node)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InternalSceneError {
    #[error("Missing node \"{0}\"")]
    MissingNode(usize),
}
