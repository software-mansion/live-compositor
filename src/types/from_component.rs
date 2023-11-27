use std::sync::Arc;

use compositor_common::scene::shader;
use compositor_common::scene::MAX_NODE_RESOLUTION;
use compositor_common::util::colors;
use compositor_render::scene;
use compositor_render::scene::Position;

use super::component::*;
use super::util::*;

impl TryFrom<Component> for scene::Component {
    type Error = TypeError;

    fn try_from(node: Component) -> Result<Self, Self::Error> {
        match node {
            Component::InputStream(input) => Ok(Self::InputStream(input.into())),
            Component::View(view) => Ok(Self::View(view.try_into()?)),
            Component::WebView(web) => Ok(Self::WebView(web.try_into()?)),
            Component::Shader(shader) => Ok(Self::Shader(shader.try_into()?)),
            Component::Image(image) => Ok(Self::Image(image.into())),
            Component::Text(text) => Ok(Self::Text(text.try_into()?)),
            Component::Tiles(tiles) => Ok(Self::Tiles(tiles.try_into()?)),
        }
    }
}

impl From<InputStream> for scene::InputStreamComponent {
    fn from(input: InputStream) -> Self {
        Self {
            id: input.id.map(Into::into),
            input_id: input.input_id.into(),
        }
    }
}

impl TryFrom<View> for scene::ViewComponent {
    type Error = TypeError;

    fn try_from(view: View) -> Result<Self, Self::Error> {
        const WIDTH_REQUIRED_MSG: &str =
            "\"View\" component with absolute positioning requires \"width\" to be specified.";
        const HEIGHT_REQUIRED_MSG: &str =
            "\"View\" component with absolute positioning requires \"height\" to be specified.";
        const VERTICAL_REQUIRED_MSG: &str =
            "\"View\" component with absolute positioning requires either \"top\" or \"bottom\" coordinate.";
        const VERTICAL_ONLY_ONE_MSG: &str = "Fields \"top\" and \"bottom\" are mutually exclusive, you can only specify one on a \"View\" component.";
        const HORIZONTAL_REQUIRED_MSG: &str =
            "Non-static \"View\" component requires either \"left\" or \"right\" coordinate.";
        const HORIZONTAL_ONLY_ONE_MSG: &str = "Fields \"left\" and \"right\" are mutually exclusive, you can only specify one on a \"View\" component.";
        let is_absolute_position = view.top.is_some()
            || view.bottom.is_some()
            || view.left.is_some()
            || view.right.is_some()
            || view.rotation.is_some();
        let position = if is_absolute_position {
            let position_vertical = match (view.top, view.bottom) {
                (Some(top), None) => scene::VerticalPosition::TopOffset(top),
                (None, Some(bottom)) => scene::VerticalPosition::BottomOffset(bottom),
                (None, None) => return Err(TypeError::new(VERTICAL_REQUIRED_MSG)),
                (Some(_), Some(_)) => return Err(TypeError::new(VERTICAL_ONLY_ONE_MSG)),
            };
            let position_horizontal = match (view.left, view.right) {
                (Some(left), None) => scene::HorizontalPosition::LeftOffset(left),
                (None, Some(right)) => scene::HorizontalPosition::RightOffset(right),
                (None, None) => return Err(TypeError::new(HORIZONTAL_REQUIRED_MSG)),
                (Some(_), Some(_)) => return Err(TypeError::new(HORIZONTAL_ONLY_ONE_MSG)),
            };
            Position::Absolute(scene::AbsolutePosition {
                width: (view
                    .width
                    .ok_or_else(|| TypeError::new(WIDTH_REQUIRED_MSG))?),
                height: (view
                    .height
                    .ok_or_else(|| TypeError::new(HEIGHT_REQUIRED_MSG))?),
                position_horizontal,
                position_vertical,
                rotation_degrees: view.rotation.unwrap_or(0.0),
            })
        } else {
            Position::Static {
                width: view.width,
                height: view.height,
            }
        };
        let direction = match view.direction {
            Some(ViewDirection::Row) => scene::ViewChildrenDirection::Row,
            Some(ViewDirection::Column) => scene::ViewChildrenDirection::Column,
            None => scene::ViewChildrenDirection::Row,
        };
        let overflow = match view.overflow {
            Some(Overflow::Visible) => scene::Overflow::Visible,
            Some(Overflow::Hidden) => scene::Overflow::Hidden,
            Some(Overflow::Fit) => scene::Overflow::Fit,
            None => scene::Overflow::Hidden,
        };
        Ok(Self {
            id: view.id.map(Into::into),
            children: view
                .children
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            direction,
            position,
            overflow,
            background_color: view
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
            transition: view.transition.map(Into::into),
        })
    }
}

impl TryFrom<Shader> for scene::ShaderComponent {
    type Error = TypeError;

    fn try_from(shader: Shader) -> Result<Self, Self::Error> {
        Ok(Self {
            id: shader.id.map(Into::into),
            shader_id: shader.shader_id.into(),
            shader_param: shader.shader_param.map(Into::into),
            size: shader.resolution.into(),
            children: shader
                .children
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<ShaderParam> for shader::ShaderParam {
    fn from(param: ShaderParam) -> Self {
        fn from_struct_field(field: ShaderParamStructField) -> shader::ShaderParamStructField {
            shader::ShaderParamStructField {
                field_name: field.field_name,
                value: field.value.into(),
            }
        }
        match param {
            ShaderParam::F32(v) => shader::ShaderParam::F32(v),
            ShaderParam::U32(v) => shader::ShaderParam::U32(v),
            ShaderParam::I32(v) => shader::ShaderParam::I32(v),
            ShaderParam::List(v) => {
                shader::ShaderParam::List(v.into_iter().map(Into::into).collect())
            }
            ShaderParam::Struct(v) => {
                shader::ShaderParam::Struct(v.into_iter().map(from_struct_field).collect())
            }
        }
    }
}

impl From<Image> for scene::ImageComponent {
    fn from(image: Image) -> Self {
        Self {
            id: image.id.map(Into::into),
            image_id: image.image_id.into(),
        }
    }
}

impl TryFrom<Text> for scene::TextComponent {
    type Error = TypeError;

    fn try_from(text: Text) -> Result<Self, Self::Error> {
        let style = match text.style {
            Some(TextStyle::Normal) => scene::TextStyle::Normal,
            Some(TextStyle::Italic) => scene::TextStyle::Italic,
            Some(TextStyle::Oblique) => scene::TextStyle::Oblique,
            None => scene::TextStyle::Normal,
        };
        let wrap = match text.wrap {
            Some(TextWrapMode::None) => scene::TextWrap::None,
            Some(TextWrapMode::Word) => scene::TextWrap::Word,
            Some(TextWrapMode::Glyph) => scene::TextWrap::Glyph,
            None => scene::TextWrap::None,
        };
        let weight = match text.weight {
            Some(TextWeight::Thin) => scene::TextWeight::Thin,
            Some(TextWeight::ExtraLight) => scene::TextWeight::ExtraLight,
            Some(TextWeight::Light) => scene::TextWeight::Light,
            Some(TextWeight::Normal) => scene::TextWeight::Normal,
            Some(TextWeight::Medium) => scene::TextWeight::Medium,
            Some(TextWeight::SemiBold) => scene::TextWeight::SemiBold,
            Some(TextWeight::Bold) => scene::TextWeight::Bold,
            Some(TextWeight::ExtraBold) => scene::TextWeight::ExtraBold,
            Some(TextWeight::Black) => scene::TextWeight::Black,
            None => scene::TextWeight::Normal,
        };
        let dimensions = match (text.width, text.height, text.max_width, text.max_height) {
            (Some(width), Some(height), _, _) => scene::TextDimensions::Fixed { width, height },
            (None, Some(_), _, _) => {
                return Err(TypeError::new(
                    "\"height\" property on a Text component can only be provided if \"width\" is also defined.",
                ));
            }
            (Some(width), None, _, max_height) => scene::TextDimensions::FittedColumn {
                width,
                max_height: max_height.unwrap_or(MAX_NODE_RESOLUTION.height as f32),
            },
            (None, None, max_width, max_height) => scene::TextDimensions::Fitted {
                max_width: max_width.unwrap_or(MAX_NODE_RESOLUTION.width as f32),
                max_height: max_height.unwrap_or(MAX_NODE_RESOLUTION.height as f32),
            },
        };
        let text = Self {
            id: text.id.map(Into::into),
            text: text.text,
            font_size: text.font_size,
            dimensions,
            line_height: text.line_height.unwrap_or(text.font_size),
            color: text
                .color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(255, 255, 255, 255)))?,
            font_family: text.font_family.unwrap_or_else(|| Arc::from("Verdana")),
            style,
            align: text.align.unwrap_or(HorizontalAlign::Left).into(),
            wrap,
            weight,
            background_color: text
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
        };
        Ok(text)
    }
}

impl TryFrom<WebView> for scene::WebViewComponent {
    type Error = TypeError;

    fn try_from(web: WebView) -> Result<Self, Self::Error> {
        Ok(Self {
            id: web.id.map(Into::into),
            children: web
                .children
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            instance_id: web.instance_id.into(),
        })
    }
}

impl TryFrom<Tiles> for scene::TilesComponent {
    type Error = TypeError;

    fn try_from(tiles: Tiles) -> Result<Self, Self::Error> {
        let result = Self {
            id: tiles.id.map(Into::into),
            children: tiles
                .children
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            width: tiles.width,
            height: tiles.height,

            background_color: tiles
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
            tile_aspect_ratio: tiles
                .tile_aspect_ratio
                .map(TryInto::try_into)
                .unwrap_or(Ok((16, 9)))?,
            margin: tiles.margin.unwrap_or(0.0),
            padding: tiles.padding.unwrap_or(0.0),
            horizontal_alignment: tiles
                .horizontal_alignment
                .unwrap_or(HorizontalAlign::Center)
                .into(),
            vertical_alignment: tiles
                .vertical_alignment
                .unwrap_or(VerticalAlign::Center)
                .into(),
        };
        Ok(result)
    }
}
