use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer};

pub const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    polygon_mode: wgpu::PolygonMode::Fill,
    topology: wgpu::PrimitiveTopology::TriangleList,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    strip_index_format: None,
    conservative: false,
    unclipped_depth: false,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
    };
}

pub struct RectangleRenderBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}

/// Vertex and index buffer that describe render area as an rectangle mapped to texture.
impl RectangleRenderBuffers {
    const VERTICES: [Vertex; 4] = [
        Vertex {
            position: [1.0, -1.0, 0.0],
            texture_coords: [1.0, 1.0],
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            texture_coords: [1.0, 0.0],
        },
        Vertex {
            position: [-1.0, 1.0, 0.0],
            texture_coords: [0.0, 0.0],
        },
        Vertex {
            position: [-1.0, -1.0, 0.0],
            texture_coords: [0.0, 1.0],
        },
    ];

    #[rustfmt::skip]
    pub const INDICES: [u16; 6] = [
        0, 1, 2,
        2, 3, 0,
    ];

    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::VERTICES),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::INDICES),
        });

        Self {
            vertex: vertex_buffer,
            index: index_buffer,
        }
    }
}

pub struct Sampler {
    _sampler: wgpu::Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Sampler {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sampler bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sampler bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }],
        });

        Self {
            _sampler: sampler,
            bind_group,
            bind_group_layout,
        }
    }
}

// TODO: This should be done with push-constants, not with a buffer
pub struct U32Uniform {
    pub buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl U32Uniform {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform u32 buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::bytes_of(&0u32),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(std::mem::size_of::<u32>() as u64),
                }),
            }],
        });
        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}
