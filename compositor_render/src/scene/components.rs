use std::{fmt::Display, sync::Arc, time::Duration};

use compositor_common::{
    renderer_spec::RendererId,
    scene::{shader::ShaderParam, InputId, Resolution},
    util::colors::RGBAColor,
};

use super::Component;

mod interpolation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId(pub Arc<str>);

impl Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct InputStreamComponent {
    pub id: Option<ComponentId>,
    pub input_id: InputId,
}

#[derive(Debug, Clone)]
pub struct ShaderComponent {
    pub id: Option<ComponentId>,
    pub children: Vec<Component>,

    pub shader_id: RendererId,
    pub shader_param: Option<ShaderParam>,
    /// Render resolution
    pub size: Resolution,
}

#[derive(Debug, Clone)]
pub struct ViewComponent {
    pub id: Option<ComponentId>,
    pub children: Vec<Component>,

    pub direction: ViewChildrenDirection,
    pub position: Position,
    pub transition: Option<Transition>,

    pub background_color: RGBAColor,
}

#[derive(Debug, Clone, Copy)]
pub struct Transition {
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Static {
        width: Option<usize>,
        height: Option<usize>,
    },
    Relative(RelativePosition),
}

#[derive(Debug, Clone, Copy)]
pub struct RelativePosition {
    pub width: usize,
    pub height: usize,
    pub position_horizontal: HorizontalPosition,
    pub position_vertical: VerticalPosition,
    pub rotation_degrees: f32,
}

#[derive(Debug, Clone)]
pub enum ViewChildrenDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalPosition {
    Top(usize),
    Bottom(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum HorizontalPosition {
    Left(usize),
    Right(usize),
}
