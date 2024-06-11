use std::sync::{atomic::AtomicBool, Arc, OnceLock};

use log::{error, info};

use super::{
    common_pipeline::plane::Plane, format::TextureFormat, texture::Texture, utils::TextureUtils,
    CreateWgpuCtxError, WgpuErrorScope,
};

static USE_GLOBAL_WGPU_CTX: AtomicBool = AtomicBool::new(false);

pub fn use_global_wgpu_ctx() {
    USE_GLOBAL_WGPU_CTX.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn global_wgpu_ctx(
    force_gpu: bool,
    features: wgpu::Features,
) -> Result<Arc<WgpuCtx>, CreateWgpuCtxError> {
    static CTX: OnceLock<Result<Arc<WgpuCtx>, CreateWgpuCtxError>> = OnceLock::new();

    CTX.get_or_init(|| Ok(Arc::new(WgpuCtx::create(force_gpu, features)?)))
        .clone()
}

#[derive(Debug)]
pub struct WgpuCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub shader_header: naga::Module,

    pub format: TextureFormat,
    pub utils: TextureUtils,

    pub uniform_bgl: wgpu::BindGroupLayout,
    pub plane: Plane,
    pub empty_texture: Texture,
}

impl WgpuCtx {
    pub fn new(force_gpu: bool, features: wgpu::Features) -> Result<Arc<Self>, CreateWgpuCtxError> {
        if USE_GLOBAL_WGPU_CTX.load(std::sync::atomic::Ordering::Relaxed) {
            global_wgpu_ctx(force_gpu, features)
        } else {
            Ok(Arc::new(Self::create(force_gpu, features)?))
        }
    }

    fn create(force_gpu: bool, features: wgpu::Features) -> Result<Self, CreateWgpuCtxError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        log_available_adapters(&instance);

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            }))
            .ok_or(CreateWgpuCtxError::NoAdapter)?;

        let adapter_info = adapter.get_info();
        info!(
            "Using {} adapter with {:?} backend",
            adapter_info.name, adapter_info.backend
        );
        if force_gpu && adapter_info.device_type != wgpu::DeviceType::Cpu {
            error!("Selected adapter is CPU based. Aborting.");
            return Err(CreateWgpuCtxError::NoAdapter);
        }
        let required_features =
            features | wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::PUSH_CONSTANTS;

        let missing_features = required_features.difference(adapter.features());
        if !missing_features.is_empty() {
            error!("Selected adapter or its driver does not support required wgpu features. Missing features: {missing_features:?}).");
            error!("You can configure some of the required features using \"LIVE_COMPOSITOR_REQUIRED_WGPU_FEATURES\" environment variable. Check https://compositor.live/docs for more.");
            return Err(CreateWgpuCtxError::NoAdapter);
        }

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                required_features,
            },
            None,
        ))?;

        let shader_header = crate::transformations::shader::validation::shader_header();

        let scope = WgpuErrorScope::push(&device);

        let format = TextureFormat::new(&device);
        let utils = TextureUtils::new(&device);

        let uniform_bgl = uniform_bind_group_layout(&device);

        let plane = Plane::new(&device);
        let empty_texture = Texture::empty(&device);

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
            uniform_bgl,
            plane,
            empty_texture,
        })
    }
}

fn uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniform bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    })
}

fn log_available_adapters(instance: &wgpu::Instance) {
    let adapters: Vec<_> = instance
        .enumerate_adapters(wgpu::Backends::all())
        .iter()
        .map(|adapter| {
            let info = adapter.get_info();
            format!("\n - {info:?}")
        })
        .collect();
    info!("Available adapters: {}", adapters.join(""))
}
