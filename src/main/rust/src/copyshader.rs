use core::prelude::*;
use core::{mem, fmt};
use core::fmt::Show;
use core::borrow::IntoCow;

use opengles::gl2;
use opengles::gl2::{GLint, GLuint, GLfloat};

use glcommon;
use glcommon::{check_gl_error, get_shader_handle, get_uniform_handle_option, Shader, GLResult, UsingDefaults, MString};
use gltexture::{Texture};
    
static TRIANGLE_VERTICES: [GLfloat; 8] = [
   -1.0,  1.0,
   -1.0, -1.0,
    1.0, -1.0,
    1.0,  1.0
];
static TEXTURE_VERTICES: [GLfloat; 8] = [
    0.0, 1.0,
    0.0, 0.0,
    1.0, 0.0,
    1.0, 1.0
];

static DEFAULT_VERTEX_SHADER: &'static str = include_str!("../includes/shaders/default_copy.vsh");
static DEFAULT_FRAGMENT_SHADER: &'static str = include_str!("../includes/shaders/default_copy.fsh");

pub struct CopyShader {
    program: GLuint,
    position_handle: GLuint,
    tex_coord_handle: GLuint,
    texture_handle: GLint,
    matrix_handle: GLint,
    texture_size_handle: Option<GLint>,
    pub source: (MString, MString),
}

impl Shader for CopyShader {
    fn new(vert: MString, frag: MString) -> GLResult<CopyShader> {
        let program = try!(glcommon::create_program(vert.as_slice(), frag.as_slice()));

        let position_option = get_shader_handle(program, "vPosition");
        let tex_coord_option = get_shader_handle(program, "vTexCoord");
        let texture_option = get_uniform_handle_option(program, "texture");
        let matrix_option = get_uniform_handle_option(program, "textureMatrix");
        let texturesize_option = get_uniform_handle_option(program, "texturesize");
        match (position_option, tex_coord_option, texture_option, matrix_option) {
            (Some(position), Some(tex_coord), Some(texture), Some(matrix)) => {
                let shader = CopyShader {
                    program: program,
                    position_handle: position,
                    tex_coord_handle: tex_coord,
                    texture_handle: texture,
                    matrix_handle: matrix,
                    texture_size_handle: texturesize_option,
                    source: (vert, frag),
                };
                logi!("created {:?}", shader);
                Ok(shader)
            }
            _ => {
                gl2::delete_program(program);
                Err("copy shader missing vPosition, vTexCoord, or texture".into_cow())
            }
        }
    }
}

impl CopyShader {
    pub fn prep(&self, texture: &Texture, matrix: &[f32]) {
        gl2::use_program(self.program);
        check_gl_error("copyshader: use_program");

        glattrib_f32!(self.position_handle, 2, TRIANGLE_VERTICES);
        glattrib_f32!(self.tex_coord_handle, 2, TEXTURE_VERTICES);

        gl2::uniform_matrix_4fv(self.matrix_handle, false, matrix);
        check_gl_error("uniform_matrix_4fv(textureMatrix)");

        self.texture_size_handle.map(|ts| {
            let (w, h) = texture.dimensions;
            gl2::uniform_2f(ts, w as f32, h as f32);
        });

        gl_bindtexture!(0, gl2::TEXTURE_2D, texture.texture, self.texture_handle as GLint);
    }
}

impl Drop for CopyShader {
    fn drop(&mut self) {
        logi!("dropping {:?}", self);
        gl2::delete_program(self.program);
    }
}

impl Show for CopyShader {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "copy shader 0x{:x}", self.program)
    }
}

impl UsingDefaults<(Option<MString>, Option<MString>)> for CopyShader {
    type Defaults = (MString, MString);
    fn maybe_init(init: (Option<MString>, Option<MString>)) -> GLResult<CopyShader> {
        let (vert, frag) = fill_defaults(init);
        Shader::new(vert, frag)
    }
    fn get_source(&self) -> &(MString, MString) { &self.source }
}

fn fill_defaults(init: (Option<MString>, Option<MString>)) -> (MString, MString) {
    let (vertopt, fragopt) = init;
    let vert = vertopt.unwrap_or_else(|| { logi!("point shader: using default vertex shader"); DEFAULT_VERTEX_SHADER.into_cow()});
    let frag = fragopt.unwrap_or_else(|| { logi!("point shader: using default fragment shader"); DEFAULT_FRAGMENT_SHADER.into_cow()});
    (vert, frag)
}
