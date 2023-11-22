use compositor_common::scene::builtin_transformations;
use compositor_common::scene::builtin_transformations::BuiltinSpec;
use compositor_common::scene::shader;
use compositor_common::scene::transition;

use super::component::*;

impl From<shader::ShaderParam> for ShaderParam {
    fn from(param: shader::ShaderParam) -> Self {
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
            BuiltinSpec::FixedPositionLayout(spec) => Self::FixedPositionLayout(spec.into()),
            BuiltinSpec::TiledLayout(_) => panic!("not supported"),
            BuiltinSpec::MirrorImage { .. } => panic!("not supported"),
            BuiltinSpec::CornersRounding { .. } => panic!("not supported"),
            BuiltinSpec::FitToResolution(_) => panic!("not supported"),
            BuiltinSpec::FillToResolution { .. } => panic!("not supported"),
            BuiltinSpec::StretchToResolution { .. } => panic!("not supported"),
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
