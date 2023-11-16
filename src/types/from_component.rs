use compositor_common::scene::shader;
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
            Component::WebRenderer(_node) => todo!(),
            Component::Shader(shader) => Ok(Self::Shader(shader.try_into()?)),
            Component::Image(_node) => todo!(),
            Component::Text(_node) => todo!(),
            Component::FixedPositionLayout(_node) => {
                todo!()
            }
            Component::TiledLayout(_node) => todo!(),
            Component::MirrorImage(_node) => todo!(),
            Component::CornersRounding(_node) => todo!(),
            Component::FitToResolution(_node) => todo!(),
            Component::FillToResolution { resolution: _ } => {
                todo!()
            }
            Component::StretchToResolution { resolution: _ } => todo!(),
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
                (Some(top), None) => scene::VerticalPosition::TopOffset(top as f32),
                (None, Some(bottom)) => scene::VerticalPosition::BottomOffset(bottom as f32),
                (None, None) => return Err(TypeError::new(VERTICAL_REQUIRED_MSG)),
                (Some(_), Some(_)) => return Err(TypeError::new(VERTICAL_ONLY_ONE_MSG)),
            };
            let position_horizontal = match (view.left, view.right) {
                (Some(left), None) => scene::HorizontalPosition::LeftOffset(left as f32),
                (None, Some(right)) => scene::HorizontalPosition::RightOffset(right as f32),
                (None, None) => return Err(TypeError::new(HORIZONTAL_REQUIRED_MSG)),
                (Some(_), Some(_)) => return Err(TypeError::new(HORIZONTAL_ONLY_ONE_MSG)),
            };
            Position::Absolute(scene::AbsolutePosition {
                width: (view
                    .width
                    .ok_or_else(|| TypeError::new(WIDTH_REQUIRED_MSG))?)
                    as f32,
                height: (view
                    .height
                    .ok_or_else(|| TypeError::new(HEIGHT_REQUIRED_MSG))?)
                    as f32,
                position_horizontal,
                position_vertical,
                rotation_degrees: view.rotation.unwrap_or(0.0),
            })
        } else {
            Position::Static {
                width: view.width.map(|v| v as f32),
                height: view.height.map(|v| v as f32),
            }
        };
        let direction = match view.direction {
            Some(ViewDirection::Row) => scene::ViewChildrenDirection::Row,
            Some(ViewDirection::Column) => scene::ViewChildrenDirection::Column,
            None => scene::ViewChildrenDirection::Row,
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
            shader_param: shader.shader_params.map(Into::into),
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
