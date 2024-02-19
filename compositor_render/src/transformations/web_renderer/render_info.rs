use bytes::Bytes;
use nalgebra_glm::Mat4;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct RenderInfo {
    is_website_texture: i32,
    _padding: [i32; 3],
    transformation_matrix: [[f32; 4]; 4],
}

impl RenderInfo {
    pub fn website() -> Self {
        Self {
            is_website_texture: 1,
            _padding: Default::default(),
            transformation_matrix: Mat4::identity().transpose().into(),
        }
    }

    pub fn source_transform(transformation_matrix: &Mat4) -> Self {
        Self {
            is_website_texture: 0,
            _padding: [0; 3],
            transformation_matrix: transformation_matrix.transpose().into(),
        }
    }

    pub fn bytes(self) -> Bytes {
        Bytes::copy_from_slice(bytemuck::cast_slice(&[self]))
    }

    pub fn size() -> u32 {
        std::mem::size_of::<RenderInfo>() as u32
    }
}
