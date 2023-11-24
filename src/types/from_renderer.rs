use compositor_common::renderer_spec;

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
        };
        Ok(Self::Shader(spec))
    }
}

impl TryFrom<WebRendererSpec> for renderer_spec::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: WebRendererSpec) -> Result<Self, Self::Error> {
        let embedding_method = match spec.embedding_method {
            Some(WebEmbeddingMethod::ChromiumEmbedding) => {
                renderer_spec::WebEmbeddingMethod::ChromiumEmbedding
            }
            Some(WebEmbeddingMethod::NativeEmbeddingOverContent) => {
                renderer_spec::WebEmbeddingMethod::NativeEmbeddingOverContent
            }
            Some(WebEmbeddingMethod::NativeEmbeddingUnderContent) => {
                renderer_spec::WebEmbeddingMethod::NativeEmbeddingUnderContent
            }
            None => renderer_spec::WebEmbeddingMethod::NativeEmbeddingOverContent,
        };

        let spec = renderer_spec::WebRendererSpec {
            instance_id: spec.instance_id.into(),
            url: spec.url,
            resolution: spec.resolution.into(),
            fallback_strategy: spec
                .fallback_strategy
                .map(Into::into)
                .unwrap_or(renderer_spec::FallbackStrategy::FallbackIfAllInputsMissing),
            embedding_method,
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
