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
    Rescaler(Rescaler),
}

/// Component representing incoming RTP stream. Specific streams can be identified
/// by an `input_id` that was part of a `RegisterInputStream` request.
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

    /// Direction defines how static children are positioned inside the View component.
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

    /// Defines how this component will behave during a scene update. This will only have an
    /// effect if previous scene already contained a View component with the same id.
    pub transition: Option<Transition>,

    /// (default="hidden") Controls what happens to content that is too big to fit into an area.
    pub overflow: Option<Overflow>,

    /// (default="#00000000") Background color in a "#RRGGBBAA" format.
    pub background_color_rgba: Option<RGBAColor>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Overflow {
    /// Components that are outside of their parent area will be rendered.
    Visible,
    /// Only render parts of the children that are inside their parent area.
    Hidden,
    /// If children component are to big to fit inside the parent resize everything inside to fit.
    ///
    /// Components that have dynamic size will be treated as if they had a size 0 when calculating
    /// scaling factor.
    ///
    /// Warning: This will resize everything inside even absolutely positioned
    /// elements. For example, if you have an element in the bottom right corner and content will
    /// be rescaled by a factor 0.5x then that component will end up in the middle of it's parent
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
pub struct Rescaler {
    // TODO: better name
    pub id: Option<ComponentId>,
    pub child: Box<Component>,

    pub mode: Option<ResizeMode>,
    pub horizontal_align: Option<HorizontalAlign>,
    pub vertical_align: Option<VerticalAlign>,

    /// Width of a component in pixels. Required when using absolute positioning.
    pub width: Option<f32>,
    /// Height of a component in pixels. Required when using absolute positioning.
    pub height: Option<f32>,

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

    /// Defines how this component will behave during a scene update. This will only have an
    /// effect if previous scene already contained a View component with the same id.
    pub transition: Option<Transition>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResizeMode {
    Fit,
    Fill,
}

/// WebView component renders a website using Chromium.
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebView {
    pub id: Option<ComponentId>,
    pub children: Option<Vec<Component>>,

    /// ID of a previously registered `WebRenderer`.
    ///
    /// Warning: You can only refer to specific instance in one Component at the time.
    pub instance_id: RendererId,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub id: Option<ComponentId>,

    /// ID of a previously registered Image.
    pub image_id: RendererId,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    pub id: Option<ComponentId>,
    pub children: Option<Vec<Component>>,

    /// ID of a previously registered Shader.
    pub shader_id: RendererId,
    /// Object that will be serialized into a `struct` and passed inside the shader as:
    /// ```wgsl
    /// @group(1) @binding(0) var<uniform>
    /// ```
    ///
    /// Note: This object's structure must match the structure defined in a shader source code.
    pub shader_param: Option<ShaderParam>,
    /// Resolution of a texture where shader will be executed.
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

    /// Width of a texture that text will be rendered on. If not provided the resulting texture
    /// will be sized based on the defined text, but limited to `max_width` value.
    pub width: Option<f32>,
    /// Height of a texture that text will be rendered on. If not provided the resulting texture
    /// will be sized based on the defined text, but limited to `max_width` value.
    ///
    /// It's an error to provide `height` if width is not defined.
    pub height: Option<f32>,
    /// (default=7682) Maximal width. Limits the width of a texture that text will be rendered on.
    /// Value is ignored if width is defined.
    pub max_width: Option<f32>,
    /// (default=4320) Maximal height. Limits the height of a texture that text will be rendered on.
    /// Value is ignored if height is defined.
    pub max_height: Option<f32>,

    /// Font size in pixels.
    pub font_size: f32,
    /// Distance between lines in pixels. Defaults to the value of the `font_size` property.
    pub line_height: Option<f32>,
    /// (default="#FFFFFFFF") Font color in `#RRGGBBAA` format.
    pub color_rgba: Option<RGBAColor>,
    /// (default="#00000000") Background color in `#RRGGBBAA` format.
    pub background_color_rgba: Option<RGBAColor>,
    /// (default="Verdana") Font family.
    ///
    /// Provide family-name for specific font. "generic-family" values like e.g. "sans-serif" will not work.
    /// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value
    pub font_family: Option<Arc<str>>,
    /// (default="normal") Font style. The selected font needs to support this specific style.
    pub style: Option<TextStyle>,
    /// (default="left") Text align.
    pub align: Option<HorizontalAlign>,
    /// (default="none") Text wrapping options.
    pub wrap: Option<TextWrapMode>,
    /// (default="normal") Font weight. The selected font needs to support this specific weight.
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
    /// Disable text wrapping. Text that does not fit inside the texture will be cut off.
    None,
    /// Wraps at a glyph level.
    Glyph,
    /// Wraps at a word level. Prevent splitting words when wrapping.
    Word,
}

/// Font weight, based on [OpenType specification](https://learn.microsoft.com/en-gb/typography/opentype/spec/os2#usweightclass).
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextWeight {
    /// Weight 100.
    Thin,
    /// Weight 200.
    ExtraLight,
    /// Weight 300.
    Light,
    /// Weight 400.
    Normal,
    /// Weight 500.
    Medium,
    /// Weight 600.
    SemiBold,
    /// Weight 700.
    Bold,
    /// Weight 800.
    ExtraBold,
    /// Weight 900.
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

    /// Width of a component in pixels.
    pub width: Option<f32>,
    /// Height of a component in pixels.
    pub height: Option<f32>,

    /// (default="#00000000") Background color in a "#RRGGBBAA" format.
    pub background_color_rgba: Option<RGBAColor>,
    /// (default="16:9") Aspect ration of a tile in "W:H" format, where W and H are integers.
    pub tile_aspect_ratio: Option<AspectRatio>,
    /// (default=0) Margin of each tile in pixels.
    pub margin: Option<f32>,
    /// (default=0) Padding on each tile in pixels.
    pub padding: Option<f32>,
    /// (default="center") Horizontal alignment of tiles.
    pub horizontal_alignment: Option<HorizontalAlign>,
    /// (default="center") Vertical alignment of tiles.
    pub vertical_alignment: Option<VerticalAlign>,
}
