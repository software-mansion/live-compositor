use nalgebra_glm::Mat4;

pub fn mat4_to_bytes(mat: &Mat4) -> bytes::Bytes {
    let mut matrices_bytes = bytes::BytesMut::new();

    let colum_based = mat.transpose();
    for el in &colum_based {
        matrices_bytes.extend_from_slice(&el.to_ne_bytes())
    }

    matrices_bytes.freeze()
}
