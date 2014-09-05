use core::prelude::*;
use core::{mem, fmt};
use core::fmt::Show;

use opengles::gl2;
use opengles::gl2::{GLint, GLuint, GLfloat};

use log::{logi,loge};

use glcommon;
use glcommon::{check_gl_error, get_shader_handle, get_uniform_handle_option, Shader};
use gltexture::{Texture};
    
static gTriangleVertices: [GLfloat, ..8] = [
   -1.0,  1.0,
   -1.0, -1.0,
    1.0, -1.0,
    1.0,  1.0
];
static gTextureVertices: [GLfloat, ..8] = [
    0.0, 1.0,
    0.0, 0.0,
    1.0, 0.0,
    1.0, 1.0
];

static default_vertex_shader: &'static str =
   "attribute vec4 vPosition;
    attribute vec4 vTexCoord;
    uniform mat4 textureMatrix;
    varying vec2 uv;
    void main() {
        uv = (textureMatrix * vTexCoord).xy;
        gl_Position = vPosition;
    }\n";

static default_fragment_shader: &'static str =
   "precision lowp float;
    uniform sampler2D texture;
    varying vec2 uv;
    void main() {
        gl_FragColor = texture2D(texture, uv);
    }\n";

pub static noalpha_fragment_shader: &'static str =
   "precision lowp float;
    uniform sampler2D texture;
    varying vec2 uv;
    void main() {
        gl_FragColor = vec4(vec3(texture2D(texture, uv)), 1.0);
    }\n";

pub struct CopyShader {
    program: GLuint,
    position_handle: GLuint,
    tex_coord_handle: GLuint,
    texture_handle: GLint,
    matrix_handle: GLint,
}

impl Shader for CopyShader {
    fn new(vertopt: Option<&str>, fragopt: Option<&str>) -> Option<CopyShader> {
        let vert = vertopt.unwrap_or_else(|| { logi("copy shader: using default vertex shader"); default_vertex_shader});
        let frag = fragopt.unwrap_or_else(|| { logi("copy shader: using default fragment shader"); default_fragment_shader});
        let program_option = glcommon::create_program(vert, frag);
        match program_option {
            None => {
                loge("could not create texture copy shader");
                None
            }
            Some(program) => {
                let position_option = get_shader_handle(program, "vPosition");
                let tex_coord_option = get_shader_handle(program, "vTexCoord");
                let texture_option = get_uniform_handle_option(program, "texture");
                let matrix_option = get_uniform_handle_option(program, "textureMatrix");
                match (position_option, tex_coord_option, texture_option, matrix_option) {
                    (Some(position), Some(tex_coord), Some(texture), Some(matrix)) => {
                        let shader = CopyShader {
                            program: program,
                            position_handle: position,
                            tex_coord_handle: tex_coord,
                            texture_handle: texture,
                            matrix_handle: matrix,
                        };
                        logi!("created {}", shader);
                        Some(shader)
                    }
                    _ => {
                        loge!("copy shader missing vPosition, vTexCoord, or texture");
                        gl2::delete_program(program);
                        None
                    }
                }
            }
        }
    }
}

impl CopyShader {
    pub fn prep(&self, texture: &Texture, matrix: &[f32]) {
        gl2::use_program(self.program);
        check_gl_error("copyshader: use_program");

        glattrib_f32!(self.position_handle, 2, gTriangleVertices);
        glattrib_f32!(self.tex_coord_handle, 2, gTextureVertices);

        gl2::uniform_matrix_4fv(self.matrix_handle, false, matrix);
        check_gl_error("uniform_matrix_4fv(textureMatrix)");

        gl_bindtexture!(0, gl2::TEXTURE_2D, texture.texture, self.texture_handle as GLint);
    }
}

impl Drop for CopyShader {
    fn drop(&mut self) {
        logi!("dropping {}", self);
        gl2::delete_program(self.program);
    }
}

impl Show for CopyShader {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "copy shader 0x{:x}", self.program)
    }
}
