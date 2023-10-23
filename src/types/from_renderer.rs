use compositor_common::{
    renderer_spec,
    scene::constraints::{self, input_count},
};

use super::renderer::*;
use super::util::*;

impl From<FallbackStrategy> for renderer_spec::FallbackStrategy {
    fn from(strategy: FallbackStrategy) -> Self {
        match strategy {
            FallbackStrategy::NeverFallback => renderer_spec::FallbackStrategy::NeverFallback,
            FallbackStrategy::FallbackIfAllInputsMissing => {
                renderer_spec::FallbackStrategy::FallbackIfAllInputsMissing
            }
            FallbackStrategy::FallbackIfAnyInputMissing => {
                renderer_spec::FallbackStrategy::FallbackIfAnyInputMissing
            }
        }
    }
}

impl TryFrom<NodeConstraints> for constraints::NodeConstraints {
    type Error = TypeError;

    fn try_from(constraints: NodeConstraints) -> Result<Self, Self::Error> {
        Ok(Self(
            constraints
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<Constraint> for constraints::Constraint {
    type Error = TypeError;

    fn try_from(constraint: Constraint) -> Result<Self, Self::Error> {
        let constraint = match constraint {
            Constraint::InputCount(constraint) => {
                match (
                    constraint.fixed_count,
                    constraint.lower_bound,
                    constraint.upper_bound,
                ) {
                    (Some(fixed_count), None, None) => {
                        Self::InputCount(input_count::InputCountConstraint::Exact { fixed_count })
                    }
                    (None, Some(lower_bound), Some(upper_bound)) => {
                        Self::InputCount(input_count::InputCountConstraint::Range {
                            lower_bound,
                            upper_bound,
                        })
                    }
                    _ => return Err(TypeError::new("\"input_count\" constraint requires either \"fixed_count\" field or both \"lower_bound\" and \"upper_bound\" fields.")),
                }
            }
        };
        Ok(constraint)
    }
}

impl TryFrom<ShaderSpec> for renderer_spec::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: ShaderSpec) -> Result<Self, Self::Error> {
        let spec = renderer_spec::ShaderSpec {
            shader_id: spec.shader_id.into(),
            source: spec.source,
            fallback_strategy: spec
                .fallback_strategy
                .map(Into::into)
                .unwrap_or(renderer_spec::FallbackStrategy::FallbackIfAllInputsMissing),
            constraints: spec
                .constraints
                .unwrap_or_else(|| {
                    NodeConstraints(vec![Constraint::InputCount(InputCountConstraint {
                        fixed_count: None,
                        lower_bound: Some(0),
                        upper_bound: Some(16),
                    })])
                })
                .try_into()?,
        };
        Ok(Self::Shader(spec))
    }
}

impl TryFrom<WebRendererSpec> for renderer_spec::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: WebRendererSpec) -> Result<Self, Self::Error> {
        let spec = renderer_spec::WebRendererSpec {
            instance_id: spec.instance_id.into(),
            url: spec.url,
            resolution: spec.resolution.into(),
            fallback_strategy: spec
                .fallback_strategy
                .map(Into::into)
                .unwrap_or(renderer_spec::FallbackStrategy::FallbackIfAllInputsMissing),
            constraints: spec
                .constraints
                .unwrap_or_else(|| {
                    NodeConstraints(vec![Constraint::InputCount(InputCountConstraint {
                        fixed_count: None,
                        lower_bound: Some(0),
                        upper_bound: Some(16),
                    })])
                })
                .try_into()?,
        };
        Ok(Self::WebRenderer(spec))
    }
}

impl TryFrom<ImageSpec> for renderer_spec::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: ImageSpec) -> Result<Self, Self::Error> {
        fn from_url_or_path(
            url: Option<String>,
            path: Option<String>,
        ) -> Result<renderer_spec::ImageSrc, TypeError> {
            match (url, path) {
                (None, None) => Err(TypeError::new(
                    "\"url\" or \"path\" field is required when registering an image.",
                )),
                (None, Some(path)) => Ok(renderer_spec::ImageSrc::LocalPath { path }),
                (Some(url), None) => Ok(renderer_spec::ImageSrc::Url { url }),
                (Some(_), Some(_)) => Err(TypeError::new(
                    "\"url\" and \"path\" fields are mutually exclusive when registering an image.",
                )),
            }
        }
        let image = match spec {
            ImageSpec::Png {
                image_id,
                url,
                path,
            } => renderer_spec::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: renderer_spec::ImageType::Png,
            },
            ImageSpec::Jpeg {
                image_id,
                url,
                path,
            } => renderer_spec::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: renderer_spec::ImageType::Jpeg,
            },
            ImageSpec::Svg {
                image_id,
                url,
                path,
                resolution,
            } => renderer_spec::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: renderer_spec::ImageType::Svg {
                    resolution: resolution.map(Into::into),
                },
            },
            ImageSpec::Gif {
                image_id,
                url,
                path,
            } => renderer_spec::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: renderer_spec::ImageType::Gif,
            },
        };
        Ok(Self::Image(image))
    }
}
