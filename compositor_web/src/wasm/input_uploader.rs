use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_render::{Frame, FrameData, FrameSet, InputId};
use wasm_bindgen::JsValue;

use super::{
    types::{FrameFormat, InputFrame, InputFrameSet},
    wgpu::pad_to_256,
};

#[derive(Default)]
pub struct InputUploader {
    textures: HashMap<String, Texture>,
}

impl InputUploader {
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input: InputFrameSet,
    ) -> Result<FrameSet<InputId>, JsValue> {
        let pts = Duration::from_millis(input.pts_ms as u64);
        let mut frames = HashMap::new();
        for frame in input.frames.entries() {
            let frame: InputFrame = frame?.try_into()?;
            self.upload_input_frame(device, queue, &frame);

            let data = match frame.format {
                FrameFormat::RgbaBytes => FrameData::Rgba8UnormWgpuTexture(
                    self.textures.get(&frame.id).unwrap().texture.clone(),
                ),
            };

            frames.insert(
                InputId(frame.id.into()),
                Frame {
                    data,
                    resolution: frame.resolution,
                    pts,
                },
            );
        }

        Ok(FrameSet { frames, pts })
    }

    fn upload_input_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &InputFrame,
    ) {
        match frame.format {
            FrameFormat::RgbaBytes => {
                let size = wgpu::Extent3d {
                    width: frame.resolution.width as u32,
                    height: frame.resolution.height as u32,
                    depth_or_array_layers: 1,
                };
                let texture = self
                    .textures
                    .entry(frame.id.clone())
                    .or_insert_with(|| Self::create_texture(device, frame));
                if size != texture.size {
                    *texture = Self::create_texture(device, frame);
                }

                queue.write_texture(
                    texture.texture.as_image_copy(),
                    &frame.data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(pad_to_256(4 * size.width)),
                        rows_per_image: Some(size.height),
                    },
                    size,
                );
            }
        }
    }

    fn create_texture(device: &wgpu::Device, frame: &InputFrame) -> Texture {
        let size = wgpu::Extent3d {
            width: frame.resolution.width as u32,
            height: frame.resolution.height as u32,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some(&format!("Input texture: {}", frame.id)),
        });

        Texture {
            size,
            texture: Arc::new(texture),
        }
    }
}

struct Texture {
    size: wgpu::Extent3d,
    texture: Arc<wgpu::Texture>,
}
