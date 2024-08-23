use std::collections::HashMap;

use compositor_render::{FrameData, FrameSet, OutputId, Resolution};
use tracing::error;
use wasm_bindgen::JsValue;
use web_sys::ImageData;

use super::{types::to_js_error, utils::pad_to_256};

#[derive(Default)]
pub struct OutputDownloader {
    buffers: HashMap<OutputId, wgpu::Buffer>,
}

impl OutputDownloader {
    pub fn download_outputs(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        outputs: FrameSet<OutputId>,
    ) -> Result<js_sys::Map, JsValue> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        for (id, frame) in outputs.frames.iter() {
            let FrameData::Rgba8UnormWgpuTexture(texture) = &frame.data else {
                panic!("Expected Rgba8UnormWgpuTexture");
            };

            let buffer = self
                .buffers
                .entry(id.clone())
                .or_insert_with(|| Self::create_buffer(device, frame.resolution));
            Self::ensure_buffer(buffer, device, frame.resolution);
            Self::copy_texture_to_buffer(texture, buffer, &mut encoder);
        }
        queue.submit(Some(encoder.finish()));

        let mut pending_downloads = Vec::with_capacity(outputs.frames.len());
        for (id, buffer) in self.buffers.iter_mut() {
            let (map_complete_sender, map_complete_receiver) = crossbeam_channel::bounded(1);
            buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    if let Err(err) = map_complete_sender.send(result) {
                        error!("channel send error: {err}")
                    }
                });
            pending_downloads.push((id.clone(), map_complete_receiver));
        }

        device.poll(wgpu::Maintain::Wait);

        let output_data = js_sys::Map::new();
        for (id, map_complete_receiver) in pending_downloads {
            map_complete_receiver.recv().unwrap().map_err(to_js_error)?;
            let resolution = outputs.frames.get(&id).unwrap().resolution;
            let buffer = self.buffers.get(&id).unwrap();
            {
                let buffer_view = buffer.slice(..).get_mapped_range();
                let data = ImageData::new_with_u8_clamped_array_and_sh(
                    wasm_bindgen::Clamped(&buffer_view),
                    resolution.width as u32,
                    resolution.height as u32,
                )?;
                output_data.set(&id.to_string().into(), &data.into());
            }

            buffer.unmap();
        }

        Ok(output_data)
    }

    fn create_buffer(device: &wgpu::Device, resolution: Resolution) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (4 * resolution.width * resolution.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    fn ensure_buffer(buffer: &mut wgpu::Buffer, device: &wgpu::Device, resolution: Resolution) {
        if buffer.size() != (4 * resolution.width * resolution.height) as u64 {
            *buffer = Self::create_buffer(device, resolution);
        }
    }

    fn copy_texture_to_buffer(
        texture: &wgpu::Texture,
        buffer: &wgpu::Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let size = texture.size();
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(pad_to_256(4 * size.width)),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );
    }
}