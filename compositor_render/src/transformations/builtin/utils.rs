use nalgebra_glm::Mat4;

pub fn mat_as_slice(mat: &Mat4) -> &[u8] {
    let mat: &[[f32; 4]; 4] = mat.as_ref();

    let byte_slice: &[u8] = unsafe {
        std::slice::from_raw_parts(
            mat.as_ptr() as *const u8,
            mat.len() * std::mem::size_of::<[f32; 4]>(),
        )
    };

    byte_slice
}
