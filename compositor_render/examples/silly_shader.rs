use std::{path::Path, process::Stdio, sync::Arc, time::Duration};

use compositor_common::{
    frame::YuvData,
    scene::{InputSpec, NodeId, OutputSpec, Resolution, SceneSpec, ShaderParam, TransformNodeSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
    Frame,
};
use compositor_render::{frame_set::FrameSet, Renderer};

fn ffmpeg_yuv_to_jpeg(
    input_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    resolution: Resolution,
) {
    std::process::Command::new("ffmpeg")
        .arg("-s")
        .arg(format!("{}x{}", resolution.width, resolution.height))
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-i")
        .arg(input_file.as_ref().as_os_str())
        .arg(output_file.as_ref().as_os_str())
        .arg("-y")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ffmpeg")
        .wait()
        .expect("wait");
}

fn ffmpeg_jpeg_to_yuv(input_file: impl AsRef<Path>, output_file: impl AsRef<Path>) {
    std::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file.as_ref().as_os_str())
        .arg(output_file.as_ref().as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn get_image(path: impl AsRef<Path>) -> Frame {
    ffmpeg_jpeg_to_yuv(&path, "input.yuv");

    let jpeg = image::open(path).expect("picture load");
    let resolution = Resolution {
        width: jpeg.width() as usize,
        height: jpeg.height() as usize,
    };

    let yuv = std::fs::read("input.yuv").expect("yuv load");
    let yuv = bytes::Bytes::from(yuv);

    let y_len = resolution.width * resolution.height;
    let yuv_data = YuvData {
        y_plane: yuv.slice(0..y_len),
        u_plane: yuv.slice(y_len..5 * y_len / 4),
        v_plane: yuv.slice(5 * y_len / 4..),
    };
    assert_eq!(yuv_data.u_plane.len(), yuv_data.v_plane.len());

    std::fs::remove_file("input.yuv").expect("rm input.yuv");

    Frame {
        data: yuv_data,
        pts: Duration::from_secs(1),
        resolution,
    }
}

fn main() {
    let frame = get_image("compositor_render/examples/crab.jpg");
    let resolution = frame.resolution;

    let renderer = Renderer::new(false).expect("create renderer");
    let shader_key = TransformationRegistryKey("silly shader".into());

    renderer
        .register_transformation(
            shader_key.clone(),
            TransformationSpec::Shader {
                source: include_str!("silly.wgsl").into(),
            },
        )
        .expect("create shader");

    let input_id = NodeId("input".into());
    let shader_id = NodeId("silly".into());
    let output_id = NodeId("output".into());

    renderer
        .update_scene(SceneSpec {
            inputs: vec![InputSpec {
                input_id: input_id.clone().into(),
                resolution,
            }],
            transforms: vec![TransformNodeSpec {
                input_pads: vec![input_id.clone()],
                node_id: shader_id.clone(),
                resolution,
                transform_params: compositor_common::scene::TransformParams::Shader {
                    shader_id: shader_key,
                    shader_params: ShaderParam::U32(42),
                },
            }],
            outputs: vec![OutputSpec {
                input_pad: shader_id,
                output_id: output_id.clone().into(),
            }],
        })
        .expect("update scene");

    let mut frame_set = FrameSet::new(Duration::from_secs_f32(std::f32::consts::FRAC_PI_2));
    frame_set.frames.insert(input_id.into(), Arc::new(frame));
    let output = renderer.render(frame_set);
    let output = output.frames.get(&output_id.into()).expect("extract frame");
    let mut output_data = Vec::with_capacity(resolution.width * resolution.height * 3 / 2);
    output_data.extend_from_slice(&output.data.y_plane);
    output_data.extend_from_slice(&output.data.u_plane);
    output_data.extend_from_slice(&output.data.v_plane);
    std::fs::write("output.yuv", output_data).expect("write");

    ffmpeg_yuv_to_jpeg("output.yuv", "output.jpg", resolution);

    std::fs::remove_file("output.yuv").expect("rm output.yuv");
}
