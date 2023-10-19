use compositor_common::scene;
use compositor_common::scene::builtin_transformations;
use compositor_common::scene::builtin_transformations::BuiltinSpec;
use compositor_common::scene::shader;
use compositor_common::scene::text_spec;
use compositor_common::scene::transition;
use compositor_common::scene::NodeSpec;

use super::node::*;
use super::Node;

impl From<NodeSpec> for Node {
    fn from(node: NodeSpec) -> Self {
        let params = match node.params {
            scene::NodeParams::WebRenderer { instance_id } => {
                NodeParams::WebRenderer(WebRenderer {
                    instance_id: instance_id.into(),
                })
            }
            scene::NodeParams::Shader {
                shader_id,
                shader_params,
                resolution,
            } => NodeParams::Shader(Shader {
                shader_id: shader_id.into(),
                shader_params: shader_params.map(Into::into),
                resolution: resolution.into(),
            }),
            scene::NodeParams::Text(spec) => NodeParams::Text(spec.into()),
            scene::NodeParams::Image { image_id } => NodeParams::Image(Image {
                image_id: image_id.into(),
            }),
            scene::NodeParams::Transition(spec) => NodeParams::Transition(spec.into()),
            scene::NodeParams::Builtin(transformation) => match transformation {
                scene::builtin_transformations::BuiltinSpec::TransformToResolution {
                    resolution,
                    strategy,
                } => NodeParams::TransformToResolution((strategy, resolution).into()),
                scene::builtin_transformations::BuiltinSpec::FixedPositionLayout(layout) => {
                    NodeParams::FixedPositionLayout(layout.into())
                }
                scene::builtin_transformations::BuiltinSpec::TiledLayout(layout) => {
                    NodeParams::TiledLayout(layout.into())
                }
                scene::builtin_transformations::BuiltinSpec::MirrorImage { mode } => {
                    NodeParams::MirrorImage(MirrorImage {
                        mode: Some(mode.into()),
                    })
                }
                scene::builtin_transformations::BuiltinSpec::CornersRounding { border_radius } => {
                    NodeParams::CornersRounding(CornersRounding {
                        border_radius: border_radius.into(),
                    })
                }
            },
        };
        Self {
            node_id: node.node_id.into(),
            input_pads: Some(node.input_pads.into_iter().map(Into::into).collect()),
            fallback_id: node.fallback_id.map(Into::into),
            params,
        }
    }
}

impl From<shader::ShaderParam> for ShaderParam {
    fn from(param: scene::shader::ShaderParam) -> Self {
        fn from_struct_field(field: shader::ShaderParamStructField) -> ShaderParamStructField {
            ShaderParamStructField {
                field_name: field.field_name,
                value: field.value.into(),
            }
        }
        match param {
            shader::ShaderParam::F32(value) => ShaderParam::F32(value),
            shader::ShaderParam::U32(value) => ShaderParam::U32(value),
            shader::ShaderParam::I32(value) => ShaderParam::I32(value),
            shader::ShaderParam::List(value) => {
                ShaderParam::List(value.into_iter().map(Into::into).collect())
            }
            shader::ShaderParam::Struct(value) => {
                ShaderParam::Struct(value.into_iter().map(from_struct_field).collect())
            }
        }
    }
}

impl From<text_spec::TextSpec> for Text {
    fn from(spec: text_spec::TextSpec) -> Self {
        let style = match spec.style {
            text_spec::Style::Normal => TextStyle::Normal,
            text_spec::Style::Italic => TextStyle::Italic,
            text_spec::Style::Oblique => TextStyle::Oblique,
        };
        let wrap = match spec.wrap {
            text_spec::Wrap::None => TextWrapMode::None,
            text_spec::Wrap::Glyph => TextWrapMode::Glyph,
            text_spec::Wrap::Word => TextWrapMode::Word,
        };
        let weight = match spec.weight {
            text_spec::Weight::Thin => TextWeight::Thin,
            text_spec::Weight::ExtraLight => TextWeight::ExtraLight,
            text_spec::Weight::Light => TextWeight::Light,
            text_spec::Weight::Normal => TextWeight::Normal,
            text_spec::Weight::Medium => TextWeight::Medium,
            text_spec::Weight::SemiBold => TextWeight::SemiBold,
            text_spec::Weight::Bold => TextWeight::Bold,
            text_spec::Weight::ExtraBold => TextWeight::ExtraBold,
            text_spec::Weight::Black => TextWeight::Black,
        };
        Self {
            content: spec.content,
            font_size: spec.font_size,
            dimensions: spec.dimensions.into(),
            line_height: spec.line_height,
            color_rgba: Some(spec.color_rgba.into()),
            background_color_rgba: Some(spec.background_color_rgba.into()),
            font_family: Some(spec.font_family),
            style: Some(style),
            align: Some(spec.align.into()),
            wrap: Some(wrap),
            weight: Some(weight),
        }
    }
}

impl From<text_spec::TextDimensions> for TextDimensions {
    fn from(dim: text_spec::TextDimensions) -> Self {
        match dim {
            text_spec::TextDimensions::Fitted {
                max_width,
                max_height,
            } => TextDimensions::Fitted {
                max_width: Some(max_width),
                max_height: Some(max_height),
            },
            text_spec::TextDimensions::FittedColumn { width, max_height } => {
                TextDimensions::FittedColumn {
                    width,
                    max_height: Some(max_height),
                }
            }
            text_spec::TextDimensions::Fixed { width, height } => {
                TextDimensions::Fixed { width, height }
            }
        }
    }
}

impl From<transition::TransitionSpec> for Transition {
    fn from(spec: transition::TransitionSpec) -> Self {
        Self {
            start: spec.start.into(),
            end: spec.end.into(),
            transition_duration_ms: spec.transition_duration.as_secs_f64() * 1000.0,
            interpolation: spec.interpolation.into(),
        }
    }
}

impl From<transition::Interpolation> for Interpolation {
    fn from(interpolation: transition::Interpolation) -> Self {
        match interpolation {
            transition::Interpolation::Linear => Interpolation::Linear,
            transition::Interpolation::Spring => Interpolation::Spring,
        }
    }
}

impl From<BuiltinSpec> for TransitionState {
    fn from(spec: BuiltinSpec) -> Self {
        match spec {
            BuiltinSpec::TransformToResolution { .. } => panic!("not supported"),
            BuiltinSpec::FixedPositionLayout(spec) => Self::FixedPositionLayout(spec.into()),
            BuiltinSpec::TiledLayout(_) => panic!("not supported"),
            BuiltinSpec::MirrorImage { .. } => panic!("not supported"),
            BuiltinSpec::CornersRounding { .. } => panic!("not supported"),
        }
    }
}

impl
    From<(
        builtin_transformations::TransformToResolutionStrategy,
        scene::Resolution,
    )> for TransformToResolution
{
    fn from(
        (strategy, resolution): (
            builtin_transformations::TransformToResolutionStrategy,
            scene::Resolution,
        ),
    ) -> Self {
        match strategy {
            builtin_transformations::TransformToResolutionStrategy::Stretch => {
                TransformToResolution::Stretch {
                    resolution: resolution.into(),
                }
            }
            builtin_transformations::TransformToResolutionStrategy::Fill => {
                TransformToResolution::Fill {
                    resolution: resolution.into(),
                }
            }
            builtin_transformations::TransformToResolutionStrategy::Fit {
                background_color_rgba,
                horizontal_alignment,
                vertical_alignment,
            } => TransformToResolution::Fit {
                resolution: resolution.into(),
                background_color_rgba: Some(background_color_rgba.into()),
                horizontal_alignment: Some(horizontal_alignment.into()),
                vertical_alignment: Some(vertical_alignment.into()),
            },
        }
    }
}

impl From<builtin_transformations::FixedPositionLayoutSpec> for FixedPositionLayout {
    fn from(spec: builtin_transformations::FixedPositionLayoutSpec) -> Self {
        fn from_texture_layout(layout: builtin_transformations::TextureLayout) -> TextureLayout {
            TextureLayout {
                top: match layout.vertical_position {
                    builtin_transformations::VerticalPosition::Top(top) => Some(top.into()),
                    builtin_transformations::VerticalPosition::Bottom(_) => None,
                },
                bottom: match layout.vertical_position {
                    builtin_transformations::VerticalPosition::Top(_) => None,
                    builtin_transformations::VerticalPosition::Bottom(bottom) => {
                        Some(bottom.into())
                    }
                },
                left: match layout.horizontal_position {
                    builtin_transformations::HorizontalPosition::Left(left) => Some(left.into()),
                    builtin_transformations::HorizontalPosition::Right(_) => None,
                },
                right: match layout.horizontal_position {
                    builtin_transformations::HorizontalPosition::Left(_) => None,
                    builtin_transformations::HorizontalPosition::Right(right) => Some(right.into()),
                },
                scale: Some(layout.scale),
                rotation: Some(layout.rotation.into()),
            }
        }
        Self {
            resolution: spec.resolution.into(),
            texture_layouts: spec
                .texture_layouts
                .into_iter()
                .map(from_texture_layout)
                .collect(),
            background_color_rgba: Some(spec.background_color_rgba.into()),
        }
    }
}

impl From<builtin_transformations::tiled_layout::TiledLayoutSpec> for TiledLayout {
    fn from(layout: builtin_transformations::tiled_layout::TiledLayoutSpec) -> Self {
        Self {
            resolution: layout.resolution.into(),
            background_color_rgba: Some(layout.background_color_rgba.into()),
            tile_aspect_ratio: Some(layout.tile_aspect_ratio),
            margin: Some(layout.margin),
            padding: Some(layout.padding),
            horizontal_alignment: Some(layout.horizontal_alignment.into()),
            vertical_alignment: Some(layout.vertical_alignment.into()),
        }
    }
}

impl From<builtin_transformations::MirrorMode> for MirrorMode {
    fn from(mode: builtin_transformations::MirrorMode) -> Self {
        match mode {
            builtin_transformations::MirrorMode::Horizontal => Self::Horizontal,
            builtin_transformations::MirrorMode::Vertical => Self::Vertical,
            builtin_transformations::MirrorMode::HorizontalAndVertical => {
                Self::HorizontalAndVertical
            }
        }
    }
}
