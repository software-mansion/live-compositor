use std::{
    str::{from_utf8, Utf8Error},
    sync::{Arc, Mutex},
};

use bytes::{Bytes, BytesMut};
use compositor_common::{scene::Resolution, transformation::ImageSpec};
use image::ImageFormat;
use resvg::{
    tiny_skia,
    usvg::{self, TreeParsing},
};

use crate::renderer::{
    texture::{NodeTexture, RGBATexture},
    RegisterTransformationCtx, RenderCtx, WgpuCtx,
};

pub struct Image {
    renderer: ImageRenderer,
}

impl Image {
    pub fn new(ctx: &RegisterTransformationCtx, spec: ImageSpec) -> Result<Self, ImageError> {
        let file = Self::download_file(spec.url())?;
        let renderer = match spec {
            ImageSpec::Png { .. } => {
                let asset = BitmapAsset::new(&ctx.wgpu_ctx, file, ImageFormat::Png)?;
                ImageRenderer::BitmapAsset(asset)
            }
            ImageSpec::Jpeg { .. } => {
                let asset = BitmapAsset::new(&ctx.wgpu_ctx, file, ImageFormat::Jpeg)?;
                ImageRenderer::BitmapAsset(asset)
            }
            ImageSpec::Svg { resolution, .. } => {
                let asset = SvgAsset::new(&ctx.wgpu_ctx, file, resolution)?;
                ImageRenderer::Svg(asset)
            }
            ImageSpec::Gif { .. } => {
                // TODO: support dynamic GIFs
                let asset = BitmapAsset::new(&ctx.wgpu_ctx, file, ImageFormat::Gif)?;
                ImageRenderer::BitmapAsset(asset)
            }
        };
        Ok(Self { renderer })
    }

    fn download_file(url: &str) -> Result<bytes::Bytes, ImageError> {
        // TODO: support local files
        let response = reqwest::blocking::get(url)?;
        let response = response.error_for_status()?;
        Ok(response.bytes()?)
    }
}

pub struct ImageNode {
    image: Arc<Image>,
    was_rendered: Mutex<bool>,
}

impl ImageNode {
    pub fn new(image: Arc<Image>) -> Self {
        Self {
            image,
            was_rendered: false.into(),
        }
    }

    pub fn render(&self, ctx: &mut RenderCtx, target: &NodeTexture) {
        // TODO: This condition is only correct for static images, it needs
        // to be refactored to support e.g. GIFs with animations
        let mut was_rendered = self.was_rendered.lock().unwrap();
        if *was_rendered {
            return;
        }

        match &self.image.renderer {
            ImageRenderer::BitmapAsset(asset) => asset.render(ctx.wgpu_ctx, target),
            ImageRenderer::Svg(asset) => asset.render(ctx.wgpu_ctx, target),
        }

        *was_rendered = true;
    }

    pub fn resolution(&self) -> Resolution {
        match &self.image.renderer {
            ImageRenderer::BitmapAsset(a) => a.resolution(),
            ImageRenderer::Svg(a) => a.resolution(),
        }
    }
}

enum ImageRenderer {
    BitmapAsset(BitmapAsset),
    Svg(SvgAsset),
}

struct BitmapAsset {
    texture: RGBATexture,
}

impl BitmapAsset {
    fn new(ctx: &WgpuCtx, data: Bytes, format: ImageFormat) -> Result<Self, image::ImageError> {
        let img = image::load_from_memory_with_format(&data, format)?;
        let texture = RGBATexture::new(
            ctx,
            Resolution {
                width: img.width() as usize,
                height: img.height() as usize,
            },
        );
        texture.upload(ctx, &img.to_rgba8());
        ctx.queue.submit([]);

        Ok(Self { texture })
    }

    fn render(&self, ctx: &WgpuCtx, target: &NodeTexture) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("copy static image asset to texture"),
            });

        let size = self.texture.size();
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &self.texture.texture().texture,
            },
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &target.rgba_texture().texture().texture,
            },
            size,
        );

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn resolution(&self) -> Resolution {
        let size = self.texture.size();
        Resolution {
            width: size.width as usize,
            height: size.height as usize,
        }
    }
}

struct SvgAsset {
    texture: RGBATexture,
}

impl SvgAsset {
    fn new(
        ctx: &WgpuCtx,
        data: Bytes,
        maybe_resolution: Option<Resolution>,
    ) -> Result<Self, SvgError> {
        let text_svg = from_utf8(&data)?;
        let tree = usvg::Tree::from_str(text_svg, &Default::default())?;
        let tree = resvg::Tree::from_usvg(&tree);
        let resolution = maybe_resolution.unwrap_or_else(|| Resolution {
            width: tree.size.width() as usize,
            height: tree.size.height() as usize,
        });

        let mut buffer = BytesMut::zeroed(resolution.width * resolution.height * 4);
        let mut pixmap = tiny_skia::PixmapMut::from_bytes(
            &mut buffer,
            resolution.width as u32,
            resolution.height as u32,
        )
        .unwrap();

        let transform = match maybe_resolution {
            Some(_) => {
                let scale_multiplier = f32::min(
                    resolution.width as f32 / tree.size.width(),
                    resolution.height as f32 / tree.size.height(),
                );
                tiny_skia::Transform::from_scale(scale_multiplier, scale_multiplier)
            }
            None => tiny_skia::Transform::default(),
        };

        tree.render(transform, &mut pixmap);

        let texture = RGBATexture::new(ctx, resolution);
        texture.upload(ctx, pixmap.data_mut());
        ctx.queue.submit([]);

        Ok(Self { texture })
    }

    fn render(&self, ctx: &WgpuCtx, target: &NodeTexture) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("copy static image asset to texture"),
            });

        let size = self.texture.size();
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &self.texture.texture().texture,
            },
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &target.rgba_texture().texture().texture,
            },
            size,
        );

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn resolution(&self) -> Resolution {
        let size = self.texture.size();
        Resolution {
            width: size.width as usize,
            height: size.height as usize,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Failed to download asset")]
    AssetDownload(#[from] reqwest::Error),

    #[error("Failed to read as bitmap")]
    FailedToReadAsBitmap(#[from] image::ImageError),

    #[error("Failed to read SVG")]
    ParsingSvgFailed(#[from] SvgError),
}

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    #[error("Invalid utf-8 content")]
    InvalidUtf8Content(#[from] Utf8Error),

    #[error("Failed to parse the svg image")]
    ParsingSvgFailed(#[from] usvg::Error),
}
