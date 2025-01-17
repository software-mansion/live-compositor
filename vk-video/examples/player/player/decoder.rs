use std::{
    io::Read,
    sync::{mpsc::SyncSender, Arc},
    time::Duration,
};

use bytes::BytesMut;
use vk_video::VulkanDevice;

use super::FrameWithPts;

pub fn run_decoder(
    tx: SyncSender<super::FrameWithPts>,
    framerate: u64,
    vulkan_device: Arc<VulkanDevice>,
    mut bytestream_reader: impl Read,
) {
    let mut decoder = vulkan_device.create_wgpu_textures_decoder().unwrap();
    let frame_interval = 1.0 / (framerate as f64);
    let mut frame_number = 0u64;
    let mut buffer = BytesMut::zeroed(4096);

    while let Ok(n) = bytestream_reader.read(&mut buffer) {
        if n == 0 {
            return;
        }

        let decoded = decoder.decode(&buffer[..n], None).unwrap();

        for f in decoded {
            let result = FrameWithPts {
                frame: f.frame,
                pts: Duration::from_secs_f64(frame_number as f64 * frame_interval),
            };

            frame_number += 1;

            if tx.send(result).is_err() {
                return;
            }
        }
    }
}
