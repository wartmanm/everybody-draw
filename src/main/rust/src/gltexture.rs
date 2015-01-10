use core::prelude::*;
use core::fmt;
use core::mem;
use core::fmt::Show;
use core::hash::sip::SipHasher;
use core::hash::Hash;

use opengles::gl2;
use opengles::gl2::GLuint;

use glcommon::{check_gl_error, GLResult, FillDefaults, Defaults};

use collections::vec::Vec;

#[deriving(PartialEq, Eq, Hash, Show, Copy)]
#[repr(i8)]
pub enum PixelFormat {
    RGBA = gl2::RGBA as i8,
    RGB = gl2::RGB as i8,
    ALPHA = gl2::ALPHA as i8,
}

impl Hash<SipHasher> for PixelFormat {
    fn hash(&self, state: &mut SipHasher) {
        unsafe {
            mem::transmute::<PixelFormat, i8>(*self).hash(state);
        }
        //(self as i8).hash(state);
    }
}
impl Eq for PixelFormat { }
impl PartialEq for PixelFormat {
    fn eq(&self, other: &PixelFormat) -> bool {
        let (selfi8, otheri8) = mem::transmute::<(PixelFormat, PixelFormat), (i8, i8)>((*self, *other));
        return selfi8 == otheri8;
    }
}

pub trait ToPixelFormat {
    fn to_pixelformat(&self) -> GLResult<PixelFormat>;
}

pub struct Texture {
    pub texture: GLuint,
    pub dimensions: (i32, i32),
}

pub struct BrushTexture {
    pub texture: Texture,
    pub source: (PixelFormat, (i32, i32), Vec<u8>),
}

impl Texture {
    pub fn new() -> Texture {
        let texture = gl2::gen_textures(1)[0];
        check_gl_error("gen_textures");
        Texture { texture: texture, dimensions: (0, 0) }
    }
    pub fn with_image(w: i32, h: i32, bytes: Option<&[u8]>, format: PixelFormat) -> Texture {
        let mut texture = Texture::new();
        texture.set_image(w, h, bytes, format);
        texture
    }

    pub fn set_image(&mut self, w: i32, h: i32, bytes: Option<&[u8]>, format: PixelFormat) {
        gl2::bind_texture(gl2::TEXTURE_2D, self.texture);
        check_gl_error("Texture.set_image bind_texture");
        gl2::tex_image_2d(gl2::TEXTURE_2D, 0, format as i32, w, h, 0, format as GLuint, gl2::UNSIGNED_BYTE, bytes);
        check_gl_error("Texture.set_image tex_image_2d");

        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_S, gl2::CLAMP_TO_EDGE as i32);
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_T, gl2::CLAMP_TO_EDGE as i32);
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MIN_FILTER, gl2::NEAREST as i32);
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MAG_FILTER, gl2::NEAREST as i32);
        check_gl_error("Texture.set_image tex_parameter_i");
        self.dimensions = (w,h);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        gl2::delete_textures([self.texture].as_slice());
        logi!("deleted {:?} texture", self.dimensions);
    }
}

impl Show for Texture {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "texture 0x{:x}, dimensions {:?}", self.texture, self.dimensions)
    }
}

impl Show for BrushTexture {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "brushtexture 0x{:x}, dimensions {:?}", self.texture.texture, self.texture.dimensions)
    }
}

impl FillDefaults<(PixelFormat, (i32, i32), Vec<u8>), (PixelFormat, (i32, i32), Vec<u8>), BrushTexture> for BrushTexture {
    fn fill_defaults(init: (PixelFormat, (i32, i32), Vec<u8>)) -> Defaults<(PixelFormat, (i32, i32), Vec<u8>), BrushTexture> {
        Defaults { val: init }
    }
}
