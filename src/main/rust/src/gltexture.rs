use core::prelude::*;
use core::fmt;
use core::fmt::Show;
use core::hash::Hash;

use opengles::gl2;
use opengles::gl2::GLuint;

use glcommon::{check_gl_error, GLResult, UsingDefaults, UsingDefaultsSafe};

use collections::vec::Vec;

#[derive(PartialEq, Eq, Hash, Show, Copy)]
#[repr(u32)]
pub enum PixelFormat {
    RGBA = gl2::RGBA,
    RGB = gl2::RGB,
    ALPHA = gl2::ALPHA,
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

impl UsingDefaultsSafe for BrushTexture { }
impl UsingDefaults<(PixelFormat, (i32, i32), Vec<u8>)> for BrushTexture {
    type Defaults = (PixelFormat, (i32, i32), Vec<u8>);
    fn maybe_init(init: (PixelFormat, (i32, i32), Vec<u8>)) -> GLResult<BrushTexture> {
        let tex = {
            let (ref format, (w, h), ref pixels) = init;
            Texture::with_image(w, h, Some(pixels.as_slice()), *format)
        };
        Ok(BrushTexture { texture: tex, source: init })
    }
    fn get_source(&self) -> &(PixelFormat, (i32, i32), Vec<u8>) { &self.source }
}
