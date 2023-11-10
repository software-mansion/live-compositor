use std::{fmt::Display, sync::Arc};

use self::layout::SizedLayoutComponent;
use self::scene_state::BaseNode;

pub use scene_state::OutputScene;
pub(crate) use scene_state::{OutputNode, SceneState};

pub use components::*;

mod components;
mod layout;
mod non_layout_components;
mod scene_state;
mod view_component;

#[derive(Debug, Clone)]
pub struct ComponentId(pub Arc<str>);

impl Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum Component {
    InputStream(InputStreamComponent),
    Shader(ShaderComponent),
    Layout(LayoutComponent),
}

#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) children: Vec<Node>,
}

#[derive(Debug)]
pub(crate) enum NodeKind {
    InputStream(InputStreamComponent),
    Shader(ShaderComponent),
    Layout(LayoutNode),
}

#[derive(Debug)]
pub(crate) struct LayoutNode {
    pub(crate) root: SizedLayoutComponent,
}

impl Component {
    pub(crate) fn width(&self) -> Option<usize> {
        match self {
            Component::InputStream(input) => input.size.map(|s| s.width),
            Component::Shader(shader) => Some(shader.size.width),
            Component::Layout(layout) => layout.width(),
        }
    }

    pub(crate) fn height(&self) -> Option<usize> {
        match self {
            Component::InputStream(input) => input.size.map(|s| s.height),
            Component::Shader(shader) => Some(shader.size.height),
            Component::Layout(layout) => layout.height(),
        }
    }

    fn base_node(&self) -> Result<BaseNode, BuildSceneError> {
        match self {
            Component::InputStream(input) => input.base_node(),
            Component::Shader(shader) => shader.base_node(),
            Component::Layout(layout) => match layout {
                LayoutComponent::View(view) => view.base_node(),
            },
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
