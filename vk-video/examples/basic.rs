#[cfg(vulkan)]
fn main() {
    use std::io::Write;

    use vk_video::Frame;

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing");

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("usage: {} FILENAME", args[0]);
        return;
    }

    let h264_bytestream = std::fs::read(&args[1]).unwrap_or_else(|_| panic!("read {}", args[1]));

    let vulkan_ctx = std::sync::Arc::new(
        vk_video::VulkanCtx::new(
            wgpu::Features::empty(),
            wgpu::Limits {
                max_push_constant_size: 128,
                ..Default::default()
            },
        )
        .unwrap(),
    );
    let mut decoder = vk_video::BytesDecoder::new(vulkan_ctx).unwrap();

    let mut output_file = std::fs::File::create("output.nv12").unwrap();

    for chunk in h264_bytestream.chunks(256) {
        let frames = decoder.decode(chunk, None).unwrap();

        for Frame { frame, .. } in frames {
            output_file.write_all(&frame).unwrap();
        }
    }
}

#[cfg(not(vulkan))]
fn main() {
    println!(
        "This crate doesn't work on your operating system, because it does not support vulkan"
    );
}
