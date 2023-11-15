use compositor_common::{
    renderer_spec::RendererId,
    scene::{shader::ShaderParam, InputId, Resolution},
    util::colors::RGBAColor,
};

use super::{Component, ComponentId};

#[derive(Debug, Clone)]
pub struct InputStreamComponent {
    pub id: Option<ComponentId>,
    pub input_id: InputId,

    // part of state, not part of API
    // TODO: separate logic into stateful and stateless components
    pub size: Option<Resolution>,
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
pub enum LayoutComponent {
    View(ViewComponent),
}

#[derive(Debug, Clone)]
pub struct ViewComponent {
    pub id: Option<ComponentId>,
    pub children: Vec<Component>,

    pub direction: ViewChildrenDirection,
    pub position: Position,

    pub background_color: RGBAColor,
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
