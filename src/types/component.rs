use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::util::*;
use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Component {
    InputStream(InputStream),
    View(View),
    WebView(WebView),
    Shader(Shader),
    Image(Image),
    Text(Text),
    Tiles(Tiles),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputStream {
    pub id: Option<ComponentId>,
    pub input_id: InputId,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub id: Option<ComponentId>,
    pub children: Option<Vec<Component>>,

    /// Width of a component in pixels. Required when using absolute positioning.
    pub width: Option<f32>,
    /// Height of a component in pixels. Required when using absolute positioning.
    pub height: Option<f32>,

    /// Direction defines how static children are positioned inside the View.
    /// "row" - Children positioned from left to right.
    /// "column" - Children positioned from top to bottom.
    pub direction: Option<ViewDirection>,

    /// Distance between the top edge of this component and the top edge of its parent.
    /// If this field is defined, then component will ignore a layout defined by its parent.
    pub top: Option<f32>,
    /// Distance between the left edge of this component and the left edge of its parent.
    /// If this field is defined, this element will be absolutely positioned, instead of being
    /// laid out by it's parent.
    pub left: Option<f32>,
    /// Distance between the bottom edge of this component and the bottom edge of its parent.
    /// If this field is defined, this element will be absolutely positioned, instead of being
    /// laid out by it's parent.
    pub bottom: Option<f32>,
    /// Distance between the right edge of this component and the right edge of its parent.
    /// If this field is defined, this element will be absolutely positioned, instead of being
    /// laid out by it's parent.
    pub right: Option<f32>,
    /// Rotation of a component in degrees. If this field is defined, this element will be
    /// absolutely positioned, instead of being laid out by it's parent.
    pub rotation: Option<f32>,

    pub transition: Option<Transition>,

    pub overflow: Option<Overflow>,

    /// Background color of a component in a "#RRGGBBAA" format. Defaults to transparent
    /// "#00000000".
    pub background_color_rgba: Option<RGBAColor>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Overflow {
    Visible,
    Hidden,
    Fit,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ViewDirection {
    Row,
    Column,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebView {
    pub id: Option<ComponentId>,
    pub children: Option<Vec<Component>>,
    pub instance_id: RendererId,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub id: Option<ComponentId>,
    pub image_id: RendererId,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    pub id: Option<ComponentId>,
    pub children: Option<Vec<Component>>,
    pub shader_id: RendererId,
    pub shader_params: Option<ShaderParam>,
    pub resolution: Resolution,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    content = "value",
    deny_unknown_fields
)]
pub enum ShaderParam {
    F32(f32),
    U32(u32),
    I32(i32),
    List(Vec<ShaderParam>),
    Struct(Vec<ShaderParamStructField>),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ShaderParamStructField {
    pub field_name: String,
    #[serde(flatten)]
    pub value: ShaderParam,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub id: Option<ComponentId>,
    pub text: Arc<str>,

    pub width: Option<f32>,
    pub height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,

    pub font_size: f32,
    pub line_height: Option<f32>,
    pub color_rgba: Option<RGBAColor>,
    pub background_color_rgba: Option<RGBAColor>,
    /// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value   
    /// use font family name, not generic family name
    pub font_family: Option<String>, // TODO: Arc<str>
    pub style: Option<TextStyle>,
    pub align: Option<HorizontalAlign>,
    pub wrap: Option<TextWrapMode>,
    pub weight: Option<TextWeight>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextWrapMode {
    None,
    Glyph,
    Word,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Interpolation {
    Linear,
    Spring,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tiles {
    pub id: Option<ComponentId>,
    pub children: Option<Vec<Component>>,

    pub width: Option<f32>,
    pub height: Option<f32>,

    pub background_color_rgba: Option<RGBAColor>,
    pub tile_aspect_ratio: Option<AspectRatio>,
    pub margin: Option<f32>,
    pub padding: Option<f32>,
    pub horizontal_alignment: Option<HorizontalAlign>,
    pub vertical_alignment: Option<VerticalAlign>,
}
