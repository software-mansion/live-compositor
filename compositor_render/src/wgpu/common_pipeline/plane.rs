use wgpu::util::DeviceExt;

use super::{Vertex, MAX_PLANES_COUNT};

#[rustfmt::skip]
const INDICES: [u16; 6] = [
    0, 1, 2,
    2, 3, 0
];

pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

#[derive(Debug)]
pub struct PlaneCache {
    // Plane with vertices with id == -1
    non_indexed_plane: Plane,
    // Planes with vertices with id 0..MAX_PLANES_COUNT
    indexed_planes: Vec<Plane>,
}

impl PlaneCache {
    pub fn new(device: &wgpu::Device) -> Self {
        let indexed_planes = (0..MAX_PLANES_COUNT)
            .map(|input_id| Plane::new(device, input_id as i32))
            .collect();

        Self {
            non_indexed_plane: Plane::new(device, -1),
            indexed_planes,
        }
    }

    /// Return plane with vertices with provided input_id
    /// Return None if input_id >= cached planes count
    pub fn plane(&self, input_id: i32) -> Option<&Plane> {
        if input_id == -1 {
            Some(&self.non_indexed_plane)
        } else {
            self.indexed_planes.get(input_id as usize)
        }
    }

    /// Returns plane with vertices with input_id = -1
    pub fn non_indexed_plane(&self) -> &Plane {
        &self.non_indexed_plane
    }
}

#[derive(Debug)]
pub struct Plane {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Plane {
    pub fn new(device: &wgpu::Device, input_id: i32) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Self::vertices(input_id)),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&INDICES),
        });

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    fn vertices(input_id: i32) -> [Vertex; 4] {
        [
            Vertex {
                position: [1.0, -1.0, 0.0],
                texture_coords: [1.0, 1.0],
                input_id,
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                texture_coords: [1.0, 0.0],
                input_id,
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                texture_coords: [0.0, 0.0],
                input_id,
            },
            Vertex {
                position: [-1.0, -1.0, 0.0],
                texture_coords: [0.0, 1.0],
                input_id,
            },
        ]
    }
}
