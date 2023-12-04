use log::error;

use super::{
    common_pipeline::plane::Planes, format::TextureFormat, shader::WgpuShader, utils::TextureUtils,
    CreateWgpuCtxError, WgpuErrorScope,
};

#[derive(Debug)]
pub struct WgpuCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub shader_header: naga::Module,

    pub format: TextureFormat,
    pub utils: TextureUtils,

    pub shader_parameters_bind_group_layout: wgpu::BindGroupLayout,
    pub planes: Planes,
}

impl WgpuCtx {
    pub fn new() -> Result<Self, CreateWgpuCtxError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            }))
            .ok_or(CreateWgpuCtxError::NoAdapter)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Video Compositor's GPU :^)"),
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,
            },
            None,
        ))?;

        let shader_header =
            naga::front::wgsl::parse_str(include_str!("./shader/shader_header.wgsl"))
                .expect("failed to parse the shader header file");

        let scope = WgpuErrorScope::push(&device);

        let format = TextureFormat::new(&device);
        let utils = TextureUtils::new(&device);

        let shader_parameters_bind_group_layout =
            WgpuShader::new_parameters_bind_group_layout(&device);

        let planes = Planes::new(&device);

        scope.pop(&device)?;

        device.on_uncaptured_error(Box::new(|e| {
            error!("wgpu error: {:?}", e);
        }));

        Ok(Self {
            device,
            queue,
            shader_header,
            format,
            utils,
            shader_parameters_bind_group_layout,
            planes,
        })
    }
}
