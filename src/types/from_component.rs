use compositor_common::scene::shader;
use compositor_common::util::colors;
use compositor_render::scene;

use super::component::*;
use super::util::*;

impl TryFrom<Component> for scene::Component {
    type Error = TypeError;

    fn try_from(node: Component) -> Result<Self, Self::Error> {
        match node {
            Component::InputStream(input) => Ok(Self::InputStream(input.into())),
            Component::View(view) => {
                Ok(Self::Layout(scene::LayoutComponent::View(view.try_into()?)))
            }
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
            size: None,
        }
    }
}

impl TryFrom<View> for scene::ViewComponent {
    type Error = TypeError;

    fn try_from(view: View) -> Result<Self, Self::Error> {
        Ok(Self {
            id: view.id.map(Into::into),
            children: view
                .children
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            width: view.width,
            height: view.height,
            direction: scene::ViewChildrenDirection::Row,
            background_color: view
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
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
