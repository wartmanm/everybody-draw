use core::prelude::*;
use core::{mem, fmt};
use core::fmt::Show;
use collections::str::{StrAllocating, IntoMaybeOwned};

use log::{logi};

use opengles::gl2;
use opengles::gl2::{GLint, GLuint};

use glcommon;
use glcommon::{check_gl_error, get_shader_handle, get_uniform_handle_option, Shader, GLResult, FillDefaults, Defaults, MString};
use point::ShaderPaintPoint;
use gltexture::Texture;

static DEFAULT_VERTEX_SHADER: &'static str = include_str!("../includes/shaders/default_point.vsh");
static DEFAULT_FRAGMENT_SHADER: &'static str = include_str!("../includes/shaders/default_point.fsh");

pub struct PointShader {
    program: GLuint,
    position_handle: GLuint,
    size_handle: Option<GLuint>,
    time_handle: Option<GLuint>,
    matrix_handle: GLint,
    texture_handle: Option<GLint>,
    color_handle: GLint,
    size_factor_handle: GLint,
    pointer_handle: Option<GLuint>,
    speed_handle: Option<GLuint>,
    distance_handle: Option<GLuint>,
    back_buffer_handle: Option<GLint>,
    texture_size_handle: GLint,
    pub source: (MString, MString),
}

impl Shader for PointShader {
    fn new(vert: MString, frag: MString) -> GLResult<PointShader> {
        let program = try!(glcommon::create_program(vert.as_slice(), frag.as_slice()));

        let position_option = get_shader_handle(program, "vPosition"); 
        let matrix_option = gl2::get_uniform_location(program, "textureMatrix");
        match (position_option, matrix_option) {
            (Some(position), matrix) if matrix != -1 => {
                let shader = PointShader {
                    program: program,
                    position_handle: position,
                    size_handle: get_shader_handle(program, "vSize"),
                    time_handle: get_shader_handle(program, "vTime"),
                    matrix_handle: matrix,
                    texture_handle: get_uniform_handle_option(program, "texture"),
                    color_handle: gl2::get_uniform_location(program, "vColor"),
                    size_factor_handle: gl2::get_uniform_location(program, "vSizeFactor"),
                    pointer_handle: get_shader_handle(program, "vPointer"),
                    speed_handle: get_shader_handle(program, "vSpeed"),
                    distance_handle: get_shader_handle(program, "vDistance"),
                    back_buffer_handle: get_uniform_handle_option(program, "backbuffer"),
                    texture_size_handle: gl2::get_uniform_location(program, "texturesize"),
                    source: (vert, frag),
                };
                logi!("created {}", shader);
                Ok(shader)
            }
            _ => {
                gl2::delete_program(program);
                Err("point shader missing vPosition or textureMatrix attribute".into_string())
            }
        }
    }
}

impl PointShader {

    pub fn prep(&self, matrix: &[f32], points: &[ShaderPaintPoint], color: [f32, ..3], brushsize: f32, brush: &Texture, backbuffer: &Texture) {
        gl2::use_program(self.program);
        check_gl_error("pointshader: use_program");

        glattrib_f32!(self.position_handle, 2, points, pos);

        self.time_handle.map(|th| {
            glattrib_f32!(th, 1, points, time);
        });

        self.size_handle.map(|sh| {
            glattrib_f32!(sh, 1, points, size);
        });

        gl2::uniform_matrix_4fv(self.matrix_handle, false, matrix);
        check_gl_error("uniform_matrix_4fv(textureMatrix)");

        self.texture_handle.map(|th| {
            gl_bindtexture!(0, gl2::TEXTURE_2D, brush.texture, th as GLint);
        });

        self.pointer_handle.map(|ph| {
            glattrib_f32!(ph, 1, points, counter);
        });

        self.speed_handle.map(|sh| {
            glattrib_f32!(sh, 2, points, speed);
        });

        self.distance_handle.map(|dh| {
            glattrib_f32!(dh, 1, points, distance);
        });

        self.back_buffer_handle.map(|bb| {
            gl_bindtexture!(1, gl2::TEXTURE_2D, backbuffer.texture, bb);
        });

        let (w, h) = backbuffer.dimensions;
        gl2::uniform_2f(self.texture_size_handle, w as f32, h as f32);

        unsafe { gl2::glUniform3fv(self.color_handle, 1, color.as_ptr() as *mut f32); }
        check_gl_error("uniform3fv");

        gl2::uniform_1f(self.size_factor_handle, brushsize);
        check_gl_error("uniform1f");
    }

}

impl Drop for PointShader {
    fn drop(&mut self) {
        logi!("dropping {}", self);
        gl2::delete_program(self.program);
    }
}

impl Show for PointShader {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "point shader 0x{:x}", self.program)
    }
}


impl FillDefaults<(Option<MString>, Option<MString>), (MString, MString), PointShader> for PointShader {
    fn fill_defaults(init: (Option<MString>, Option<MString>)) -> Defaults<(MString, MString), PointShader> {
        let (vertopt, fragopt) = init;
        let vert = vertopt.unwrap_or_else(|| { logi("point shader: using default vertex shader"); DEFAULT_VERTEX_SHADER.into_maybe_owned()});
        let frag = fragopt.unwrap_or_else(|| { logi("point shader: using default fragment shader"); DEFAULT_FRAGMENT_SHADER.into_maybe_owned()});
        Defaults { val: (vert, frag) }
    }
}

