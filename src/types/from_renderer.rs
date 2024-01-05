use compositor_render::image;
use compositor_render::shader;
use compositor_render::web_renderer;

use super::renderer::*;
use super::util::*;

impl TryFrom<ShaderSpec> for compositor_render::RendererSpec {
    type Error = TypeError;

    fn try_from(spec: ShaderSpec) -> Result<Self, Self::Error> {
        let spec = shader::ShaderSpec {
            shader_id: spec.shader_id.into(),
            source: spec.source.into(),
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
                web_renderer::WebEmbeddingMethod::ChromiumEmbedding
            }
            Some(WebEmbeddingMethod::NativeEmbeddingOverContent) => {
                web_renderer::WebEmbeddingMethod::NativeEmbeddingOverContent
            }
            Some(WebEmbeddingMethod::NativeEmbeddingUnderContent) => {
                web_renderer::WebEmbeddingMethod::NativeEmbeddingUnderContent
            }
            None => web_renderer::WebEmbeddingMethod::NativeEmbeddingOverContent,
        };

        let spec = web_renderer::WebRendererSpec {
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
        ) -> Result<image::ImageSource, TypeError> {
            match (url, path) {
                (None, None) => Err(TypeError::new(
                    "\"url\" or \"path\" field is required when registering an image.",
                )),
                (None, Some(path)) => Ok(image::ImageSource::LocalPath { path }),
                (Some(url), None) => Ok(image::ImageSource::Url { url }),
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
            } => image::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: image::ImageType::Png,
            },
            ImageSpec::Jpeg {
                image_id,
                url,
                path,
            } => image::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: image::ImageType::Jpeg,
            },
            ImageSpec::Svg {
                image_id,
                url,
                path,
                resolution,
            } => image::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: image::ImageType::Svg {
                    resolution: resolution.map(Into::into),
                },
            },
            ImageSpec::Gif {
                image_id,
                url,
                path,
            } => image::ImageSpec {
                src: from_url_or_path(url, path)?,
                image_id: image_id.into(),
                image_type: image::ImageType::Gif,
            },
        };
        Ok(Self::Image(image))
    }
}
