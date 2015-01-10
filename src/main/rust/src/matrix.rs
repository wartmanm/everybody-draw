use core::prelude::*;
use core::fmt;
use core::fmt::Show;

pub type Matrix = [f32; 16];

#[repr(i32)]
#[deriving(Copy, Show, PartialEq, Eq)]
pub enum Rotation {
    Rotation0 = 0,
    Rotation90 = 1,
    Rotation180 = 2,
    Rotation270 = 3,
}

impl Show for Rotation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", match self {
            Rotation0    => "Rotation0",
            Rotation90   => "Rotation90",
            Rotation180  => "Rotation180",
            Rotation270  => "Rotation270",
        })
    }
}


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

pub fn fit_inside(srcdimensions: (i32, i32), targetdimensions: (i32, i32), rotation: Rotation) -> Matrix {
    use matrix::Rotation::*;
    logi!("using rotation {:?}", rotation);
    let (tw, th) = targetdimensions;
    let (w, h) = {
        let (srcw, srch) = srcdimensions;
        //(srcw, srch)
        let (w, h) = match rotation {
            Rotation0  | Rotation180 => (srcw, srch),
            Rotation90 | Rotation270 => (srch, srcw),
        };
        (w, h)
        //let (offsetX, offsetY) = (tw / 2f32, th / 2f32);
    };
    //let (offsetX, offsetY) = (tw / 2f32, th / 2f32);

    let (widthratio, heightratio) = ((tw as f32 / w as f32), (th as f32 / h as f32));
    // fit inside
    let ratio = if heightratio > widthratio { heightratio } else { widthratio };
    // account for gl's own scaling
    let (glratiox, glratioy) = (widthratio / ratio, heightratio / ratio);

    match rotation {
        Rotation0   => [ glratiox,                 0f32,                    0f32, 0f32,
                         0f32,                    -glratioy,                0f32, 0f32,
                         0f32,                     0f32,                    1f32, 0f32,
                        (1f32 - glratiox) / 2f32, (1f32 + glratioy) / 2f32, 0f32, 0f32],

        Rotation180 => [-glratiox,                 0f32,                    0f32, 0f32,
                         0f32,                     glratioy,                0f32, 0f32,
                         0f32,                     0f32,                    1f32, 0f32,
                        (1f32 + glratiox) / 2f32, (1f32 - glratioy) / 2f32, 0f32, 0f32],

        _           => [ 0f32,                    -glratiox,                0f32, 0f32,
                        -glratioy,                 0f32,                    0f32, 0f32,
                         0f32,                     0f32,                    1f32, 0f32,
                        (1f32 + glratiox) / 2f32, (1f32 + glratioy) / 2f32, 0f32, 0f32],

        //Rotation270 => [ 0f32,                    -glratiox,                0f32, 0f32,
                         //glratioy,                 0f32,                    0f32, 0f32,
                         //0f32,                     0f32,                    1f32, 0f32,
                        //(0f32 - glratioy) / 2f32, (0f32 + glratiox) / 2f32, 0f32, 0f32],

    }


    // rotation90, shifted left and compressed (no w/h flipping)
    /* rotation90
        _           => [ 0f32,                    -glratiox,                0f32, 0f32,
                        -glratioy,                 0f32,                    0f32, 0f32,
                         0f32,                     0f32,                    1f32, 0f32,
                        (1f32 + glratioy) / 2f32, (0f32 + glratiox) / 2f32, 0f32, 0f32],
                        */
}
