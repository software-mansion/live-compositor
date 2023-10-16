use std::time::Duration;

use anyhow::anyhow;
use compositor_common::{
    scene::{
        self,
        builtin_transformations::{
            self, tiled_layout::TiledLayoutSpec, BuiltinSpec, FixedPositionLayoutSpec,
        },
        shader,
        text_spec::{self, TextSpec},
        transition, NodeSpec, MAX_NODE_RESOLUTION,
    },
    util::colors::{self, RGBAColor},
};

use super::node::*;
use super::util::*;

impl TryFrom<Node> for NodeSpec {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let params = match node.params {
            NodeParams::WebRenderer(node) => node.into(),
            NodeParams::Shader(node) => node.into(),
            NodeParams::Image(node) => node.into(),
            NodeParams::Text(node) => node.try_into()?,
            NodeParams::Transition(node) => node.try_into()?,
            NodeParams::TransformToResolution(node) => scene::NodeParams::Builtin {
                transformation: node.try_into()?,
            },
            NodeParams::FixedPositionLayout(node) => scene::NodeParams::Builtin {
                transformation: node.try_into()?,
            },
            NodeParams::TiledLayout(node) => scene::NodeParams::Builtin {
                transformation: node.try_into()?,
            },
            NodeParams::MirrorImage(node) => scene::NodeParams::Builtin {
                transformation: node.into(),
            },
            NodeParams::CornersRounding(node) => scene::NodeParams::Builtin {
                transformation: node.try_into()?,
            },
        };
        let spec = Self {
            node_id: node.node_id.into(),
            input_pads: node
                .input_pads
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            fallback_id: node.fallback_id.map(Into::into),
            params,
        };
        Ok(spec)
    }
}

impl From<WebRenderer> for scene::NodeParams {
    fn from(node: WebRenderer) -> Self {
        Self::WebRenderer {
            instance_id: node.instance_id.into(),
        }
    }
}

impl From<Shader> for scene::NodeParams {
    fn from(node: Shader) -> Self {
        Self::Shader {
            shader_id: node.shader_id.into(),
            shader_params: node.shader_params.map(Into::into),
            resolution: node.resolution.into(),
        }
    }
}

impl From<ShaderParam> for scene::shader::ShaderParam {
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

impl From<Image> for scene::NodeParams {
    fn from(node: Image) -> Self {
        Self::Image {
            image_id: node.image_id.into(),
        }
    }
}

impl TryFrom<Text> for scene::NodeParams {
    type Error = anyhow::Error;

    fn try_from(node: Text) -> Result<Self, Self::Error> {
        let style = match node.style {
            Some(TextStyle::Normal) => text_spec::Style::Normal,
            Some(TextStyle::Italic) => text_spec::Style::Italic,
            Some(TextStyle::Oblique) => text_spec::Style::Oblique,
            None => text_spec::Style::Normal,
        };
        let wrap = match node.wrap {
            Some(TextWrapMode::None) => text_spec::Wrap::None,
            Some(TextWrapMode::Word) => text_spec::Wrap::Word,
            Some(TextWrapMode::Glyph) => text_spec::Wrap::Glyph,
            None => text_spec::Wrap::None,
        };
        let weight = match node.weight {
            Some(TextWeight::Thin) => text_spec::Weight::Thin,
            Some(TextWeight::ExtraLight) => text_spec::Weight::ExtraLight,
            Some(TextWeight::Light) => text_spec::Weight::Light,
            Some(TextWeight::Normal) => text_spec::Weight::Normal,
            Some(TextWeight::Medium) => text_spec::Weight::Medium,
            Some(TextWeight::SemiBold) => text_spec::Weight::SemiBold,
            Some(TextWeight::Bold) => text_spec::Weight::Bold,
            Some(TextWeight::ExtraBold) => text_spec::Weight::ExtraBold,
            Some(TextWeight::Black) => text_spec::Weight::Black,
            None => text_spec::Weight::Normal,
        };
        let dimensions = match node.dimensions {
            TextDimensions::Fitted {
                max_width,
                max_height,
            } => text_spec::TextDimensions::Fitted {
                max_width: max_width.unwrap_or(MAX_NODE_RESOLUTION.width as u32),
                max_height: max_height.unwrap_or(MAX_NODE_RESOLUTION.height as u32),
            },
            TextDimensions::FittedColumn { width, max_height } => {
                text_spec::TextDimensions::FittedColumn {
                    width,
                    max_height: max_height.unwrap_or(MAX_NODE_RESOLUTION.height as u32),
                }
            }
            TextDimensions::Fixed { width, height } => {
                text_spec::TextDimensions::Fixed { width, height }
            }
        };
        let text = Self::Text(TextSpec {
            content: node.content,
            font_size: node.font_size,
            dimensions,
            line_height: Some(node.line_height.unwrap_or(node.font_size)), // TODO: remove Some
            color_rgba: node
                .color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(255, 255, 255, 255)))?,
            font_family: node.font_family.unwrap_or_else(|| String::from("Verdana")),
            style,
            align: node.align.unwrap_or(HorizontalAlign::Left).into(),
            wrap,
            weight,
            background_color_rgba: node
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
        });
        Ok(text)
    }
}

impl TryFrom<TransformToResolution> for BuiltinSpec {
    type Error = anyhow::Error;

    fn try_from(node: TransformToResolution) -> Result<Self, Self::Error> {
        let result = match node {
            TransformToResolution::Stretch { resolution } => Self::TransformToResolution {
                resolution: resolution.into(),
                strategy: builtin_transformations::TransformToResolutionStrategy::Stretch,
            },
            TransformToResolution::Fill { resolution } => Self::TransformToResolution {
                resolution: resolution.into(),
                strategy: builtin_transformations::TransformToResolutionStrategy::Fill,
            },
            TransformToResolution::Fit {
                resolution,
                background_color_rgba,
                horizontal_alignment,
                vertical_alignment,
            } => Self::TransformToResolution {
                resolution: resolution.into(),
                strategy: builtin_transformations::TransformToResolutionStrategy::Fit {
                    background_color_rgba: background_color_rgba
                        .map(TryInto::try_into)
                        .unwrap_or(Ok(RGBAColor(0, 0, 0, 0)))?,
                    horizontal_alignment: horizontal_alignment
                        .unwrap_or(HorizontalAlign::Center)
                        .into(),
                    vertical_alignment: vertical_alignment.unwrap_or(VerticalAlign::Center).into(),
                },
            },
        };
        Ok(result)
    }
}

impl TryFrom<Transition> for scene::NodeParams {
    type Error = anyhow::Error;

    fn try_from(node: Transition) -> Result<Self, Self::Error> {
        let result = Self::Transition(transition::TransitionSpec {
            start: node.start.try_into()?,
            end: node.end.try_into()?,
            transition_duration: Duration::try_from_secs_f64(node.transition_duration_ms / 1000.0)?,
            interpolation: node.interpolation.into(),
        });
        Ok(result)
    }
}

impl TryFrom<TransitionState> for BuiltinSpec {
    type Error = anyhow::Error;

    fn try_from(state: TransitionState) -> Result<Self, Self::Error> {
        match state {
            TransitionState::FixedPositionLayout(state) => state.try_into(),
        }
    }
}

impl From<Interpolation> for transition::Interpolation {
    fn from(interpolation: Interpolation) -> Self {
        match interpolation {
            Interpolation::Linear => Self::Linear,
            Interpolation::Spring => Self::Spring,
        }
    }
}

impl TryFrom<FixedPositionLayout> for BuiltinSpec {
    type Error = anyhow::Error;

    fn try_from(node: FixedPositionLayout) -> Result<Self, Self::Error> {
        let result = Self::FixedPositionLayout(FixedPositionLayoutSpec {
            resolution: node.resolution.into(),
            texture_layouts: node
                .texture_layouts
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            background_color_rgba: node
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
        });
        Ok(result)
    }
}

impl TryFrom<TextureLayout> for builtin_transformations::TextureLayout {
    type Error = anyhow::Error;

    fn try_from(value: TextureLayout) -> Result<Self, Self::Error> {
        match value {
                TextureLayout {
                    top: None,
                    bottom: None,
                    ..
                } => return Err(anyhow!("Each entry in texture_layouts in transformation \"fixed_position_layout\" requires either bottom or top coordinate.")),
                TextureLayout {
                    top: Some(_),
                    bottom: Some(_),
                    ..
                } => return Err(anyhow!("Fields \"top\" and \"bottom\" are mutually exclusive, you can only specify one in texture layout in \"fixed_position_layout\" transformation.")),
                _ => (),
            };
        match value {
                TextureLayout {
                    left: None,
                    right: None,
                    ..
                } => return Err(anyhow!("Each entry in texture_layouts in transformation \"fixed_position_layout\" requires either right or left coordinate.")),
                TextureLayout {
                    left: Some(_),
                    right: Some(_),
                    ..
                } => return Err(anyhow!("Fields \"left\" and \"right\" are mutually exclusive, you can only specify one in texture layout in \"fixed_position_layout\" transformation.")),
                _ => (),
            };

        Ok(Self {
            top: value.top.map_or(Ok(None), |v| v.try_into().map(Some))?,
            bottom: value.bottom.map_or(Ok(None), |v| v.try_into().map(Some))?,
            left: value.left.map_or(Ok(None), |v| v.try_into().map(Some))?,
            right: value.right.map_or(Ok(None), |v| v.try_into().map(Some))?,
            scale: value.scale.unwrap_or(1.0),
            rotation: value.rotation.unwrap_or(Degree(0.0)).into(),
        })
    }
}

impl TryFrom<TiledLayout> for BuiltinSpec {
    type Error = anyhow::Error;

    fn try_from(layout: TiledLayout) -> Result<Self, Self::Error> {
        let result = Self::TiledLayout(TiledLayoutSpec {
            resolution: layout.resolution.into(),
            background_color_rgba: layout
                .background_color_rgba
                .map(TryInto::try_into)
                .unwrap_or(Ok(colors::RGBAColor(0, 0, 0, 0)))?,
            tile_aspect_ratio: layout.tile_aspect_ratio.unwrap_or((16, 9)),
            margin: layout.margin.unwrap_or(0),
            padding: layout.padding.unwrap_or(0),
            horizontal_alignment: layout
                .horizontal_alignment
                .unwrap_or(HorizontalAlign::Center)
                .into(),
            vertical_alignment: layout
                .vertical_alignment
                .unwrap_or(VerticalAlign::Center)
                .into(),
        });
        Ok(result)
    }
}

impl From<MirrorImage> for BuiltinSpec {
    fn from(node: MirrorImage) -> Self {
        let mode = match node.mode {
            Some(MirrorMode::Horizontal) => builtin_transformations::MirrorMode::Horizontal,
            Some(MirrorMode::Vertical) => builtin_transformations::MirrorMode::Vertical,
            Some(MirrorMode::HorizontalAndVertical) => {
                builtin_transformations::MirrorMode::HorizontalAndVertical
            }
            None => builtin_transformations::MirrorMode::Horizontal,
        };
        Self::MirrorImage { mode }
    }
}

impl TryFrom<CornersRounding> for BuiltinSpec {
    type Error = anyhow::Error;

    fn try_from(node: CornersRounding) -> Result<Self, Self::Error> {
        let result = Self::CornersRounding {
            border_radius: node.border_radius.try_into()?,
        };
        Ok(result)
    }
}
