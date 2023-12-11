use super::renderer::*;
use super::util::*;

impl From<FallbackStrategy> for compositor_render::FallbackStrategy {
    fn from(strategy: FallbackStrategy) -> Self {
        match strategy {
            FallbackStrategy::NeverFallback => compositor_render::FallbackStrategy::NeverFallback,
            FallbackStrategy::FallbackIfAllInputsMissing => {
                compositor_render::FallbackStrategy::FallbackIfAllInputsMissing
            }
            FallbackStrategy::FallbackIfAnyInputMissing => {
                compositor_render::FallbackStrategy::FallbackIfAnyInputMissing
            }
        }
    }
}

impl TryFrom<ShaderSpec> for compositor_render::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: ShaderSpec) -> Result<Self, Self::Error> {
        let spec = compositor_render::ShaderSpec {
            shader_id: spec.shader_id.into(),
            source: spec.source,
            fallback_strategy: compositor_render::FallbackStrategy::FallbackIfAllInputsMissing,
        };
        Ok(Self::Shader(spec))
    }
}

impl TryFrom<WebRendererSpec> for compositor_render::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: WebRendererSpec) -> Result<Self, Self::Error> {
        let embedding_method = match spec.embedding_method {
            Some(WebEmbeddingMethod::ChromiumEmbedding) => {
                compositor_render::WebEmbeddingMethod::ChromiumEmbedding
            }
            Some(WebEmbeddingMethod::NativeEmbeddingOverContent) => {
                compositor_render::WebEmbeddingMethod::NativeEmbeddingOverContent
            }
            Some(WebEmbeddingMethod::NativeEmbeddingUnderContent) => {
                compositor_render::WebEmbeddingMethod::NativeEmbeddingUnderContent
            }
            None => compositor_render::WebEmbeddingMethod::NativeEmbeddingOverContent,
        };

        let spec = compositor_render::WebRendererSpec {
            instance_id: spec.instance_id.into(),
            url: spec.url,
            resolution: spec.resolution.into(),
            fallback_strategy: compositor_render::FallbackStrategy::FallbackIfAllInputsMissing,
            embedding_method,
        };
        Ok(Self::WebRenderer(spec))
    }
}

impl TryFrom<ImageSpec> for compositor_render::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: ImageSpec) -> Result<Self, Self::Error> {
        fn from_url_or_path(
            url: Option<String>,
            path: Option<String>,
        ) -> Result<compositor_render::ImageSrc, TypeError> {
            match (url, path) {
                (None, None) => Err(TypeError::new(
                    "\"url\" or \"path\" field is required when registering an image.",
                )),
                (None, Some(path)) => Ok(compositor_render::ImageSrc::LocalPath { path }),
                (Some(url), None) => Ok(compositor_render::ImageSrc::Url { url }),
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
            } => compositor_render::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: compositor_render::ImageType::Png,
            },
            ImageSpec::Jpeg {
                image_id,
                url,
                path,
            } => compositor_render::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: compositor_render::ImageType::Jpeg,
            },
            ImageSpec::Svg {
                image_id,
                url,
                path,
                resolution,
            } => compositor_render::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: compositor_render::ImageType::Svg {
                    resolution: resolution.map(Into::into),
                },
            },
            ImageSpec::Gif {
                image_id,
                url,
                path,
            } => compositor_render::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: compositor_render::ImageType::Gif,
            },
        };
        Ok(Self::Image(image))
    }
}
