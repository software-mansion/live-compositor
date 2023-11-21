use std::{fmt::Display, sync::Arc, time::Duration};

use compositor_common::{
    renderer_spec::RendererId,
    scene::{shader::ShaderParam, InputId},
    util::colors::RGBAColor,
};

use super::Component;

mod convert;
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

    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct ImageComponent {
    pub id: Option<ComponentId>,
    pub image_id: RendererId,
}

#[derive(Debug, Clone)]
pub struct ViewComponent {
    pub id: Option<ComponentId>,
    pub children: Vec<Component>,

    pub direction: ViewChildrenDirection,
    pub position: Position,
    pub transition: Option<Transition>,
    pub overflow: Overflow,

    pub background_color: RGBAColor,
}

#[derive(Debug, Clone, Copy)]
pub enum Overflow {
    Visible,
    Hidden,
    Fit,
}

#[derive(Debug, Clone, Copy)]
pub struct Transition {
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Static {
        width: Option<f32>,
        height: Option<f32>,
    },
    Absolute(AbsolutePosition),
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AbsolutePosition {
    pub width: f32,
    pub height: f32,
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
    TopOffset(f32),
    BottomOffset(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum HorizontalPosition {
    LeftOffset(f32),
    RightOffset(f32),
}
