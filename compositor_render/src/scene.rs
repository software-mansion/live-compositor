use std::time::Duration;

use self::input_stream_component::InputStreamComponentState;
use self::layout::LayoutComponentState;
use self::scene_state::{BaseNode, BuildStateTreeCtx};
use self::shader_component::ShaderComponentState;

use compositor_common::scene::{OutputId, Resolution};

pub(crate) use layout::LayoutNode;
pub(crate) use scene_state::{OutputNode, SceneState};
pub(crate) use shader_component::ShaderComponentParams;

pub use components::*;

mod components;
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
    View(ViewComponent),
}

/// Stateful version of a `Component`. Represents the same element as
/// `Component`, but additionally it has its own state that can be used
/// keep track of transition or to preserve some information from
/// a previous scene update.
#[derive(Debug, Clone)]
enum ComponentState {
    InputStream(InputStreamComponentState),
    Shader(ShaderComponentState),
    Layout(LayoutComponentState),
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
    Layout(LayoutNode),
}

impl ComponentState {
    fn width(&self, pts: Duration) -> Option<usize> {
        match self {
            ComponentState::InputStream(input) => input.size.map(|s| s.width),
            ComponentState::Shader(shader) => Some(shader.component.size.width),
            ComponentState::Layout(layout) => match layout.position(pts) {
                Position::Static { width, .. } => width,
                Position::Relative(position) => Some(position.width),
            },
        }
    }

    fn height(&self, pts: Duration) -> Option<usize> {
        match self {
            ComponentState::InputStream(input) => input.size.map(|s| s.height),
            ComponentState::Shader(shader) => Some(shader.component.size.height),
            ComponentState::Layout(layout) => match layout.position(pts) {
                Position::Static { height, .. } => height,
                Position::Relative(position) => Some(position.height),
            },
        }
    }

    fn base_node(&self) -> Result<BaseNode, BuildSceneError> {
        match self {
            ComponentState::InputStream(input) => input.base_node(),
            ComponentState::Shader(shader) => shader.base_node(),
            ComponentState::Layout(layout) => match layout {
                LayoutComponentState::View(view) => view.base_node(),
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
    fn state_component(self, ctx: &BuildStateTreeCtx) -> ComponentState {
        match self {
            Component::InputStream(input) => input.state_component(ctx),
            Component::Shader(shader) => shader.state_component(ctx),
            Component::View(view) => view.state_component(ctx),
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
