use core::prelude::*;
use core::{mem, fmt};
use core::fmt::Show;

use log::{logi, loge};

use opengles::gl2;
use opengles::gl2::{GLint, GLuint};

use glcommon;
use glcommon::{check_gl_error, get_shader_handle, get_uniform_handle_option, Shader};
use point::ShaderPaintPoint;
use gltexture::Texture;

static default_vertex_shader: &'static str =
   "precision lowp float;
    uniform mat4 textureMatrix;
    attribute float vSize;
    attribute float vTime;
    attribute vec4 vPosition;
    attribute float vPointer;
    attribute float vSpeed;
    attribute float vDistance;
    uniform vec3 vColor;
    varying float time;
    varying float size;
    varying vec3 color;
    varying vec2 position;
  
    void main() {
        time = vTime;
        float tmpSize = vSize * 1500.0;
        size = clamp(tmpSize, 7.5, 60.0);
        color = vec3(1.0, 1.0, 0.0);
        gl_PointSize = 30.0;
        gl_Position = (textureMatrix * vPosition);
        position = vec2(textureMatrix * vPosition);
    }";
static default_fragment_shader: &'static str =
   "precision lowp float;
    varying float time;
    varying float size;
    varying vec3 color;
    uniform sampler2D texture;
    uniform sampler2D backbuffer;
    void main() {
        float ctime = clamp(time, 0.0, 1.0);
        float csize = clamp(size, 0.0, 1.0);
        float alpha = texture2D(texture, gl_PointCoord).a;
        gl_FragColor = vec4(color, alpha);
    }";

pub struct PointShader {
    program: GLuint,
    positionHandle: GLuint,
    sizeHandle: Option<GLuint>,
    timeHandle: Option<GLuint>,
    matrixHandle: GLint,
    textureHandle: Option<GLint>,
    colorHandle: GLint,
    pointerHandle: Option<GLuint>,
    speedHandle: Option<GLuint>,
    distanceHandle: Option<GLuint>,
    backBufferHandle: Option<GLint>,
    textureSizeHandle: GLint,
}

impl Shader for PointShader {
    fn new(vertopt: Option<&str>, fragopt: Option<&str>) -> Option<PointShader> {
        let vert = vertopt.unwrap_or_else(|| { logi("point shader: using default vertex shader"); default_vertex_shader});
        let frag = fragopt.unwrap_or_else(|| { logi("point shader: using default fragment shader"); default_fragment_shader});
        let programOption = glcommon::create_program(vert, frag);
        match programOption {
            None => {
                loge("could not create point shader");
                None
            }
            Some(program) => {
                let positionOption = get_shader_handle(program, "vPosition"); 
                let matrixOption = gl2::get_uniform_location(program, "textureMatrix");
                match (positionOption, matrixOption) {
                    (Some(position), matrix) if matrix != -1 => {
                        let shader = PointShader {
                            program: program,
                            positionHandle: position,
                            sizeHandle: get_shader_handle(program, "vSize"),
                            timeHandle: get_shader_handle(program, "vTime"),
                            matrixHandle: matrix,
                            textureHandle: get_uniform_handle_option(program, "texture"),
                            colorHandle: gl2::get_uniform_location(program, "vColor"),
                            pointerHandle: get_shader_handle(program, "vPointer"),
                            speedHandle: get_shader_handle(program, "vSpeed"),
                            distanceHandle: get_shader_handle(program, "vDistance"),
                            backBufferHandle: get_uniform_handle_option(program, "backbuffer"),
                            textureSizeHandle: gl2::get_uniform_location(program, "texturesize"),
                        };
                        logi!("created {}", shader);
                        Some(shader)
                    }
                    _ => {
                        loge("point shader missing vPosition or textureMatrix attribute");
                        gl2::delete_program(program);
                        None
                    }
                }
            }
        }
    }
}

impl PointShader {

    pub fn prep(&self, matrix: &[f32], points: &[ShaderPaintPoint], color: [f32, ..3], brush: &Texture, backbuffer: &Texture) {
        gl2::use_program(self.program);
        check_gl_error("pointshader: use_program");

        glattrib_f32!(self.positionHandle, 2, points, pos);

        self.timeHandle.map(|th| {
            glattrib_f32!(th, 1, points, time);
        });

        self.sizeHandle.map(|sh| {
            glattrib_f32!(sh, 1, points, size);
        });

        gl2::uniform_matrix_4fv(self.matrixHandle, false, matrix);
        check_gl_error("uniform_matrix_4fv(textureMatrix)");

        self.textureHandle.map(|th| {
            gl_bindtexture!(0, gl2::TEXTURE_2D, brush.texture, th as GLint);
        });

        self.pointerHandle.map(|ph| {
            glattrib_f32!(ph, 1, points, counter);
        });

        self.speedHandle.map(|sh| {
            glattrib_f32!(sh, 1, points, speed);
        });

        self.distanceHandle.map(|dh| {
            glattrib_f32!(dh, 1, points, distance);
        });

        self.backBufferHandle.map(|bb| {
            gl_bindtexture!(1, gl2::TEXTURE_2D, backbuffer.texture, bb);
        });

        let (w, h) = backbuffer.dimensions;
        gl2::uniform_2f(self.textureSizeHandle, w as f32, h as f32);

        unsafe { gl2::glUniform3fv(self.colorHandle, 3, color.as_ptr() as *mut f32); }
        check_gl_error("uniform3fv");
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

