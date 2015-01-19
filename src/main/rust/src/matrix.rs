pub type Matrix = [f32, ..16];

/// copied from android.opengl.matrix
/// intended for framebuffers, which range from (-1, -1) to (1, 1), and not textures, which range
/// from (0, 0) to (1, 1)
#[allow(dead_code)]
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix {
    let (r_width, r_height, r_depth) = (1f32 / (right - left), 1f32 / (top - bottom), 1f32 / (far - near));
    let (x, y, z) = (2f32 * r_width, 2f32 * r_height, 2f32 * r_depth);
    let (tx, ty, tz) = (-(right + left) * r_width, -(top + bottom) * r_height, -(far + near) / r_depth);
    [ x,    0f32,  0f32,  0f32,
      0f32,    y,  0f32,  0f32,
      0f32, 0f32,     z,  0f32,
      tx,     ty,    tz,  1f32,]
}

pub static IDENTITY: Matrix =
    [1f32, 0f32, 0f32, 0f32,
     0f32, 1f32, 0f32, 0f32,
     0f32, 0f32, 1f32, 0f32,
     0f32, 0f32, 0f32, 1f32,];

pub fn log(matrix: &[f32]) -> ::collections::string::String {
    format!("[[{:-5.3}, {:-5.3}, {:-5.3}, {:-5.3}]\n [{:-5.3}, {:-5.3}, {:-5.3}, {:-5.3}]\n [{:-5.3}, {:-5.3}, {:-5.3}, {:-5.3}]\n [{:-5.3}, {:-5.3}, {:-5.3}, {:-5.3}]]",
          matrix[0], matrix[1], matrix[2], matrix[3],
          matrix[4], matrix[5], matrix[6], matrix[7],
          matrix[8], matrix[9], matrix[10], matrix[11],
          matrix[12], matrix[13], matrix[14], matrix[15],
    )
}
