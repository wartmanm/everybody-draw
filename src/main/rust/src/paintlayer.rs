use core::prelude::*;

use opengles::gl2;
use opengles::gl2::GLuint;

use copyshader::CopyShader;
use gltexture::Texture;
use pointshader::PointShader;
use gltexture::PixelFormat;

pub struct TextureTarget {
    pub framebuffer: GLuint,
    pub texture: Texture,
}

pub struct PaintLayer<'a> {
    pub copyshader: Option<&'a CopyShader>,
    pub pointshader: Option<&'a PointShader>,
    pub target: TextureTarget,
    pub pointidx: i32,
}

pub struct CompletedLayer<'a, 'b> {
    pub copyshader: &'a CopyShader,
    pub pointshader: &'a PointShader,
    pub target: &'b TextureTarget,
}

impl TextureTarget {
    pub fn new(w: i32, h: i32, format: PixelFormat) -> TextureTarget {
        let framebuffer = gl2::gen_framebuffers(1)[0];
        let texture = Texture::with_image(w, h, None, format);

        gl2::bind_framebuffer(gl2::FRAMEBUFFER, framebuffer);
        gl2::framebuffer_texture_2d(gl2::FRAMEBUFFER, gl2::COLOR_ATTACHMENT0, gl2::TEXTURE_2D, texture.texture, 0);
        gl2::clear_color(0f32, 0f32, 0f32, 0f32);
        gl2::clear(gl2::COLOR_BUFFER_BIT);
        TextureTarget { framebuffer: framebuffer, texture: texture }
    }
}

impl Drop for TextureTarget {
    fn drop(&mut self) {
        // should drop texture automatically?
        gl2::delete_frame_buffers([self.framebuffer].as_slice());
        debug_logi!("deleted texturetarget: {:?} framebuffer {}", self.texture.dimensions, self.framebuffer);
    }
}

impl<'a> PaintLayer<'a> {
    pub fn new(dimensions: (i32, i32), copyshader: Option<&'a CopyShader>, pointshader: Option<&'a PointShader>, pointidx: i32) -> PaintLayer<'a> {
        let (w, h) = dimensions;
        PaintLayer {
            copyshader: copyshader,
            pointshader: pointshader,
            target: TextureTarget::new(w, h, PixelFormat::RGBA),
            pointidx: pointidx,
        }
    }

    pub fn complete<'b: 'a>(&self, basecopyshader: &'b CopyShader, basepointshader: &'b PointShader) -> CompletedLayer {
        CompletedLayer {
            copyshader: self.copyshader.unwrap_or(basecopyshader),
            pointshader: self.pointshader.unwrap_or(basepointshader),
            target: &self.target,
        }
    }
}
