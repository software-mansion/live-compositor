use std::time::Duration;

use self::image_component::StatefulImageComponent;
use self::input_stream_component::StatefulInputStreamComponent;
use self::layout::StatefulLayoutComponent;
use self::scene_state::{BuildStateTreeCtx, IntermediateNode};
use self::shader_component::StatefulShaderComponent;

use compositor_common::scene::{OutputId, Resolution};

pub(crate) use layout::LayoutNode;
pub(crate) use scene_state::{OutputNode, SceneState};
pub(crate) use shader_component::ShaderComponentParams;

pub use components::*;

mod components;
mod image_component;
mod input_stream_component;
mod layout;
mod scene_state;
mod shader_component;
mod view_component;

#[derive(Debug, Clone)]
pub struct OutputScene {
    pub output_id: OutputId,
    pub root: Component,
    pub resolution: Resolution,
}

#[derive(Debug, Clone)]
pub enum Component {
    InputStream(InputStreamComponent),
    Shader(ShaderComponent),
    Image(ImageComponent),
    View(ViewComponent),
}

/// Stateful version of a `Component`. Represents the same element as
/// `Component`, but additionally it has its own state that can be used
/// keep track of transition or to preserve some information from
/// a previous scene update.
#[derive(Debug, Clone)]
enum StatefulComponent {
    InputStream(StatefulInputStreamComponent),
    Shader(StatefulShaderComponent),
    Image(StatefulImageComponent),
    Layout(StatefulLayoutComponent),
}

/// Defines a tree structure that is a base to construct a `RenderGraph`.
/// Each `prams` element defines a parameters to construct a `RenderNode`
/// and `children` define connections between them.
///
/// In most cases each `Node` will be used to construct a RenderNode, but
/// in some cases multiple Nodes might be reduced to just one RenderNode
/// e.g. `NodeParams::InputStream` for the same input stream might be present
/// multiple times inside the tree, but it will result in only one `RenderNode`
/// in the `RenderGraph`
#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) params: NodeParams,
    pub(crate) children: Vec<Node>,
}

/// Set of params used to construct a `RenderNode`.
#[derive(Debug)]
pub(crate) enum NodeParams {
    InputStream(InputStreamComponent),
    Shader(ShaderComponentParams),
    Image(ImageComponent),
    Layout(LayoutNode),
}

impl StatefulComponent {
    fn width(&self, pts: Duration) -> Option<f32> {
        match self {
            StatefulComponent::InputStream(input) => input.size.map(|s| s.width),
            StatefulComponent::Shader(shader) => Some(shader.component.size.width),
            StatefulComponent::Image(image) => image.size.map(|s| s.width),
            StatefulComponent::Layout(layout) => match layout.position(pts) {
                Position::Static { width, .. } => width,
                Position::Absolute(position) => Some(position.width),
            },
        }
    }

    fn height(&self, pts: Duration) -> Option<f32> {
        match self {
            StatefulComponent::InputStream(input) => input.size.map(|s| s.height),
            StatefulComponent::Shader(shader) => Some(shader.component.size.height),
            StatefulComponent::Image(image) => image.size.map(|s| s.height),
            StatefulComponent::Layout(layout) => match layout.position(pts) {
                Position::Static { height, .. } => height,
                Position::Absolute(position) => Some(position.height),
            },
        }
    }

    fn intermediate_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        match self {
            StatefulComponent::InputStream(input) => input.intermediate_node(),
            StatefulComponent::Shader(shader) => shader.intermediate_node(),
            StatefulComponent::Image(image) => image.intermediate_node(),
            StatefulComponent::Layout(layout) => match layout {
                StatefulLayoutComponent::View(view) => view.intermediate_node(),
            },
        }
    }
}

impl Component {
    /// Recursively convert `Component` tree provided by a user into a
    /// `ComponentState` tree. `ComponentState` includes all the information
    /// from `Component`, but additionally it has it's own state. When calculating
    /// initial value of that state, the component has access to state of that
    /// component from before scene update.
    fn stateful_component(self, ctx: &BuildStateTreeCtx) -> StatefulComponent {
        match self {
            Component::InputStream(input) => input.stateful_component(ctx),
            Component::Shader(shader) => shader.stateful_component(ctx),
            Component::View(view) => view.stateful_component(ctx),
            Component::Image(image) => image.stateful_component(ctx),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildSceneError {
    #[error("\"{component}\" that is a child of an non-layout component e.g. \"Shader\", \"WebView\" need to have known size. {msg}")]
    UnknownDimensionsForLayoutNodeRoot {
        component: &'static str,
        msg: String,
    },
}
