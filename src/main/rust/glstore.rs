/// DrawObjectList interns shaders, brushes, and scripts, returning references that can be stored
/// in the event queue.
/// TODO: scripts
/// TODO: serialization
/// TODO: further backing store, caching init objs by sha1 or so
/// TODO: free shaders + textures on gl pause
/// TODO: cleanup, deduplication

use core::prelude::*;
use collections::vec::Vec;
use collections::string::String;
use collections::str::StrAllocating;
use collections::MutableSeq;
use collections::slice::CloneableVector;
use copyshader::CopyShader;
use gltexture::{PixelFormat, Texture};
use pointshader::PointShader;
use glcommon::Shader;

pub struct ShaderInit<T> {
    shader: Option<T>,
    vert: Option<String>,
    frag: Option<String>,
}

pub struct DrawObjects {
    copyshaders: Vec<ShaderInit<CopyShader>>,
    pointshaders: Vec<ShaderInit<PointShader>>,
    brushes: Vec<Texture>,
}

//pub enum DrawObject {
    //CopyShaderObj(ShaderInit<CopyShader>),
    //PointShaderObj(ShaderInit<PointShader>),
    //BrushObj(BrushInit),
//}

pub struct DrawObjectList {
    copyshaderlist: Vec<ShaderInit<CopyShader>>,
    pointshaderlist: Vec<ShaderInit<PointShader>>,
    brushlist: Vec<BrushInit>,
}

pub struct DrawObjectIndex<T>(i32);

impl<T: Shader> ShaderInit<T> {
    pub fn get(&self) -> &T {
        match self.shader {
            Some(x) => &x,
            None => {
                let (vert, frag) = (self.vert.map(|x|x.as_slice()), self.frag.map(|x|x.as_slice()));
                let shader = Shader::new(vert, frag).unwrap();
                self.shader = Some(shader);
                &shader
            }
        }
    }
    pub fn new(vert: Option<&str>, frag: Option<&str>) -> Option<ShaderInit<T>> {
        let shaderopt: Option<T> = Shader::new(vert, frag);
        shaderopt.map(|shader| {
            let (vertstr, fragstr) = (vert.map(|x| x.to_owned()), frag.map(|x| x.to_owned()));
            ShaderInit { shader: Some(shader), vert: vertstr, frag: fragstr }
        })
    }
}

pub struct BrushInit {
    format: PixelFormat,
    dimensions: (i32, i32),
    pixels: Vec<u8>,
    texture: Option<Texture>,
}

impl BrushInit {
    pub fn get(&self) -> &Texture {
        match self.texture {
            Some(x) => &x,
            None => {
                let (w,h) = self.dimensions;
                let texture = Texture::with_image(w, h, Some(self.pixels.as_slice()), self.format);
                self.texture = Some(texture);
                &texture
            }
        }
    }

    pub fn new(w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> BrushInit {
        BrushInit { format: format, dimensions: (w,h), pixels: pixels.to_vec(), texture: None }
    }
}

impl DrawObjectList {
    pub fn new() -> DrawObjectList {
        DrawObjectList {
            copyshaderlist: Vec::new(),
            pointshaderlist: Vec::new(),
            brushlist: Vec::new(),
        }
    }

    pub fn push_copyshader(&self, shader: ShaderInit<CopyShader>) -> DrawObjectIndex<CopyShader> {
        self.copyshaderlist.push(shader);
        DrawObjectIndex((self.copyshaderlist.len() - 1) as i32)
    }
    pub fn push_pointshader(&self, shader: ShaderInit<PointShader>) -> DrawObjectIndex<PointShader> {
        self.pointshaderlist.push(shader);
        DrawObjectIndex((self.copyshaderlist.len() - 1) as i32)
    }
    pub fn push_brush(&self, brush: BrushInit) -> DrawObjectIndex<Texture> {
        self.brushlist.push(brush);
        DrawObjectIndex((self.brushlist.len() - 1) as i32)
    }

    pub fn get_copyshader(&self, i: DrawObjectIndex<CopyShader>) -> &CopyShader {
        let DrawObjectIndex(idx) = i;
        self.copyshaderlist.get(idx as uint).get()
    }
    pub fn get_pointshader(&self, i: DrawObjectIndex<PointShader>) -> &PointShader {
        let DrawObjectIndex(idx) = i;
        self.pointshaderlist.get(idx as uint).get()
    }
    pub fn get_brush(&self, i: DrawObjectIndex<Texture>) -> &Texture {
        let DrawObjectIndex(idx) = i;
        self.brushlist.get(idx as uint).get()
    }
}
