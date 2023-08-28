use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, BufferSlice};

pub const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    polygon_mode: wgpu::PolygonMode::Fill,
    topology: wgpu::PrimitiveTopology::TriangleList,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    strip_index_format: None,
    conservative: false,
    unclipped_depth: false,
};

pub const MAX_TEXTURES_COUNT: u32 = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
    pub input_id: i32,
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Sint32],
    };

    const fn empty() -> Self {
        Vertex {
            position: [0.0, 0.0, 0.0],
            texture_coords: [0.0, 0.0],
            input_id: 0,
        }
    }
}

pub struct InputTexturesPlanes {
    inputs_vertices: Buffer,
    inputs_indices: Buffer,
    no_inputs_vertices: Buffer,
    no_inputs_indices: Buffer,
}

macro_rules! const_vertices {
    ($textures_count:expr) => {{
        let mut vertices = [Vertex::empty(); 4 * $textures_count as usize];

        let mut input_id = 0;
        while input_id < $textures_count {
            vertices[input_id as usize * 4] = Vertex {
                position: [1.0, -1.0, 0.0],
                texture_coords: [1.0, 1.0],
                input_id: input_id as i32,
            };

            vertices[input_id as usize * 4 + 1] = Vertex {
                position: [1.0, 1.0, 0.0],
                texture_coords: [1.0, 0.0],
                input_id: input_id as i32,
            };

            vertices[input_id as usize * 4 + 2] = Vertex {
                position: [-1.0, 1.0, 0.0],
                texture_coords: [0.0, 0.0],
                input_id: input_id as i32,
            };

            vertices[input_id as usize * 4 + 3] = Vertex {
                position: [-1.0, -1.0, 0.0],
                texture_coords: [0.0, 1.0],
                input_id: input_id as i32,
            };

            input_id += 1;
        }

        vertices
    }};
}

macro_rules! const_indices {
    ($textures_count:expr) => {{
        let mut indices = [0u16; 6 * $textures_count as usize];

        let mut i = 0;
        while i < $textures_count {
            indices[6 * i as usize] = (4 * i) as u16;
            indices[6 * i as usize + 1] = (4 * i + 1) as u16;
            indices[6 * i as usize + 2] = (4 * i + 2) as u16;
            indices[6 * i as usize + 3] = (4 * i + 2) as u16;
            indices[6 * i as usize + 4] = (4 * i + 3) as u16;
            indices[6 * i as usize + 5] = (4 * i) as u16;
            i += 1;
        }

        indices
    }};
}

/// Vertex and index buffer that describe render area as an rectangle mapped to texture.
impl InputTexturesPlanes {
    /// Vertices of texture planes passed to the vertex shader.
    /// Each plane has 4 vertices
    const INPUTS_VERTICES: [Vertex; 4 * MAX_TEXTURES_COUNT as usize] =
        const_vertices!(MAX_TEXTURES_COUNT);

    /// Indexes vertices of texture planes passed to vertex shader.
    /// Describes which vertices combine triangles.
    /// Each texture plane contain 2 triangles - 6 indices
    const INPUTS_INDICES: [u16; 6 * MAX_TEXTURES_COUNT as usize] =
        const_indices!(MAX_TEXTURES_COUNT);

    /// In case of no input texture, vertex shader receives plane
    /// with 4 vertices with input id -1. This allows using shaders without
    /// any input textures - e.g. shaders generating some texture
    /// based on uniform parameters.
    const NO_INPUT_VERTICES: [Vertex; 4] = [
        Vertex {
            position: [1.0, -1.0, 0.0],
            texture_coords: [1.0, 1.0],
            input_id: -1,
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            texture_coords: [1.0, 0.0],
            input_id: -1,
        },
        Vertex {
            position: [-1.0, 1.0, 0.0],
            texture_coords: [0.0, 0.0],
            input_id: -1,
        },
        Vertex {
            position: [-1.0, -1.0, 0.0],
            texture_coords: [0.0, 1.0],
            input_id: -1,
        },
    ];

    #[rustfmt::skip]
    const NO_INPUT_INDICES: [u16; 6] = [
        0, 1, 2,
        2, 3, 0
    ];

    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    pub fn new(device: &wgpu::Device) -> Self {
        let inputs_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::INPUTS_VERTICES),
        });

        let inputs_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::INPUTS_INDICES),
        });

        let no_inputs_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&Self::NO_INPUT_VERTICES),
            });

        let no_inputs_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::NO_INPUT_INDICES),
        });

        Self {
            inputs_vertices: inputs_vertex_buffer,
            inputs_indices: inputs_index_buffer,
            no_inputs_vertices: no_inputs_vertex_buffer,
            no_inputs_indices: no_inputs_index_buffer,
        }
    }

    pub fn vertices(&self, input_textures_count: u32) -> BufferSlice {
        if input_textures_count == 0 {
            self.no_inputs_vertices.slice(..)
        } else {
            let vertex_buffer_len =
                4 * input_textures_count as u64 * std::mem::size_of::<Vertex>() as u64;
            self.inputs_vertices.slice(..vertex_buffer_len)
        }
    }

    pub fn indices(&self, input_textures_count: u32) -> BufferSlice {
        if input_textures_count == 0 {
            self.no_inputs_indices.slice(..)
        } else {
            let index_buffer_len =
                6 * input_textures_count as u64 * std::mem::size_of::<u16>() as u64;
            self.inputs_indices.slice(..index_buffer_len)
        }
    }

    pub fn indices_len(input_textures_count: u32) -> u32 {
        if input_textures_count == 0 {
            6
        } else {
            input_textures_count * 6
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
