use wgpu::{util::DeviceExt, Buffer, BufferSlice};

use super::{Vertex, MAX_TEXTURES_COUNT};

/// Abstraction for buffers, holding vertices and indices of
/// 2D planes, on which textures are rendered
pub struct Surfaces {
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

/// In case of no input texture, vertex shader receives plane
/// with 4 vertices with input id -1. This allows using shaders without
/// any input textures - e.g. shaders generating some texture
/// based on uniform parameters.
const SINGLE_PLANE_VERTICES: [Vertex; 4] = [
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
const SINGLE_PLANE_INDICES: [u16; 6] = [
    0, 1, 2,
    2, 3, 0
];

pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

/// Vertex and index buffer that describe render area as an rectangle mapped to texture.
impl Surfaces {
    /// Vertices of texture 2D planes passed to the vertex shader.
    /// Each plane has 4 vertices
    const ALL_PLANES_VERTICES: [Vertex; 4 * MAX_TEXTURES_COUNT as usize] =
        const_vertices!(MAX_TEXTURES_COUNT);

    /// Indexes vertices of texture 2D planes passed to vertex shader.
    /// Describes which vertices combine triangles.
    /// Each texture plane contain 2 triangles - 6 indices
    const ALL_PLANES_INDICES: [u16; 6 * MAX_TEXTURES_COUNT as usize] =
        const_indices!(MAX_TEXTURES_COUNT);

    pub fn new(device: &wgpu::Device) -> Self {
        let inputs_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::ALL_PLANES_VERTICES),
        });

        let inputs_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::ALL_PLANES_INDICES),
        });

        let no_inputs_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&SINGLE_PLANE_VERTICES),
            });

        let no_inputs_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&SINGLE_PLANE_INDICES),
        });

        Self {
            inputs_vertices: inputs_vertex_buffer,
            inputs_indices: inputs_index_buffer,
            no_inputs_vertices: no_inputs_vertex_buffer,
            no_inputs_indices: no_inputs_index_buffer,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, input_textures_count: u32) {
        render_pass.set_vertex_buffer(0, self.vertices(input_textures_count));

        render_pass.set_index_buffer(self.indices(input_textures_count), INDEX_FORMAT);

        render_pass.draw_indexed(0..Self::indices_len(input_textures_count), 0, 0..1);
    }

    fn vertices(&self, input_textures_count: u32) -> BufferSlice {
        if input_textures_count == 0 {
            self.no_inputs_vertices.slice(..)
        } else {
            let vertex_buffer_len =
                4 * input_textures_count as u64 * std::mem::size_of::<Vertex>() as u64;
            self.inputs_vertices.slice(..vertex_buffer_len)
        }
    }

    fn indices(&self, input_textures_count: u32) -> BufferSlice {
        if input_textures_count == 0 {
            self.no_inputs_indices.slice(..)
        } else {
            let index_buffer_len =
                6 * input_textures_count as u64 * std::mem::size_of::<u16>() as u64;
            self.inputs_indices.slice(..index_buffer_len)
        }
    }

    fn indices_len(input_textures_count: u32) -> u32 {
        if input_textures_count == 0 {
            6
        } else {
            input_textures_count * 6
        }
    }
}

pub struct SingleSurface {
    vertices: Buffer,
    indices: Buffer,
}

impl SingleSurface {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&SINGLE_PLANE_VERTICES),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&SINGLE_PLANE_INDICES),
        });

        Self {
            vertices: vertex_buffer,
            indices: index_buffer,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));

        render_pass.set_index_buffer(self.indices.slice(..), INDEX_FORMAT);

        render_pass.draw_indexed(0..SINGLE_PLANE_INDICES.len() as u32, 0, 0..1);
    }
}
