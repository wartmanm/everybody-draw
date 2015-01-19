extern crate opengles;
use core::prelude::*;
use collections::vec::Vec;
use collections::str::StrAllocating;
use collections::{Mutable, MutableSeq};

use log::logi;

use opengles::gl2;
use opengles::gl2::{GLuint, GLenum, GLubyte};

use glcommon::{Shader, check_gl_error, GLResult};
use glpoint::{MotionEventConsumer};
use point::ShaderPaintPoint;
use pointshader::PointShader;
use paintlayer::{TextureTarget, CompletedLayer};
use copyshader::*;
use gltexture;
use gltexture::Texture;
use matrix;
use eglinit;
use luascript::LuaScript;
use paintlayer::PaintLayer;
use lua_callbacks::LuaCallbackType;
use lua_geom::do_interpolate_lua;
use drawevent::Events;


static DRAW_INDEXES: [GLubyte, ..6] = [
    0, 1, 2,
    0, 2, 3
];

//#[deriving(FromPrimitive)]
#[repr(i32)]
#[allow(non_camel_case_types, dead_code)]
pub enum AndroidBitmapFormat {
    ANDROID_BITMAP_FORMAT_NONE      = 0,
    ANDROID_BITMAP_FORMAT_RGBA_8888 = 1,
    ANDROID_BITMAP_FORMAT_RGB_565   = 4,
    ANDROID_BITMAP_FORMAT_RGBA_4444 = 7,
    ANDROID_BITMAP_FORMAT_A_8       = 8,
}


/// struct for storage of data that stays on rust side
/// should probably be given a meaningful name like PaintContext, but w/e
pub struct GLInit<'a> {
    #[allow(dead_code)]
    pub dimensions: (i32, i32),
    pub paintstate: PaintState<'a>,
    targetdata: TargetData,
    pub points: Vec<Vec<ShaderPaintPoint>>,
}

pub struct TargetData {
    targets: [TextureTarget, ..2],
    current_target: u8,
}

pub struct PaintState<'a> {
    pub pointshader: Option<&'a PointShader>,
    pub animshader: Option<&'a CopyShader>,
    pub copyshader: Option<&'a CopyShader>,
    pub brush: Option<&'a Texture>,
    pub interpolator: Option<&'a LuaScript>,
    pub layers: Vec<PaintLayer<'a>>,
}

impl<'a> PaintState<'a> {
    pub fn new() -> PaintState<'a> {
        PaintState {
            pointshader: None,
            animshader: None,
            copyshader: None,
            brush: None,
            interpolator: None,
            layers: Vec::new(),
        }
    }
}

fn print_gl_string(name: &str, s: GLenum) {
    let glstr = gl2::get_string(s);
    logi!("GL {} = {}\n", name, glstr);
}

fn perform_copy(dest_framebuffer: GLuint, source_texture: &Texture, shader: &CopyShader, matrix: &[f32]) -> () {
    gl2::bind_framebuffer(gl2::FRAMEBUFFER, dest_framebuffer);
    check_gl_error("bound framebuffer");
    shader.prep(source_texture, matrix);
    gl2::draw_elements(gl2::TRIANGLES, DRAW_INDEXES.len() as i32, gl2::UNSIGNED_BYTE, Some(DRAW_INDEXES.as_slice()));
    check_gl_error("drew elements");
}

fn draw_layer(layer: CompletedLayer, matrix: &[f32], color: [f32, ..3]
              , brush: &Texture, back_buffer: &Texture, points: &[ShaderPaintPoint]) {
    if points.len() > 0 {
        gl2::bind_framebuffer(gl2::FRAMEBUFFER, layer.target.framebuffer);
        layer.pointshader.prep(matrix.as_slice(), points, color, brush, back_buffer);
        gl2::draw_arrays(gl2::POINTS, 0, points.len() as i32);
        check_gl_error("draw_arrays");
    }
}

impl TargetData {
    fn get_current_texturetarget<'a>(&'a self) -> &'a TextureTarget {
        &self.targets[self.current_target as uint]
    }

    fn get_current_texturesource<'a> (&'a self) -> &'a TextureTarget {
        &self.targets[(self.current_target ^ 1) as uint]
    }

    fn get_texturetargets<'a> (&'a self) -> (&'a TextureTarget, &'a TextureTarget) {
        (self.get_current_texturetarget(), self.get_current_texturesource())
    }
}

impl<'a> GLInit<'a> {
    pub fn draw_image(&mut self, w: i32, h: i32, pixels: &[u8]) -> () {
        logi("drawing image...");
        let target = self.targetdata.get_current_texturetarget();
        let (tw, th) = target.texture.dimensions;
        let heightratio = th as f32 / h as f32;
        let widthratio = tw as f32 / w as f32;
        // fit inside
        let ratio = if heightratio > widthratio { heightratio } else { widthratio };
        // account for gl's own scaling
        let (glratiox, glratioy) = (widthratio / ratio, heightratio / ratio);

        let matrix = [ glratiox,                 0f32,                    0f32, 0f32,
                       0f32,                    -glratioy,                0f32, 0f32,
                       0f32,                     0f32,                    1f32, 0f32,
                      (1f32 - glratiox) / 2f32, (1f32 + glratioy) / 2f32, 0f32, 0f32];
        logi!("drawing with ratio: {:5.3f}, glratio {:5.3f}, {:5.3f}, matrix:\n{}", ratio, glratiox, glratioy, matrix::log(matrix.as_slice()));

        self.paintstate.copyshader.map(|shader| {
            let intexture = Texture::with_image(w, h, Some(pixels), gltexture::RGBA);
            check_gl_error("creating texture");
            perform_copy(target.framebuffer, &intexture, shader, matrix.as_slice());
        });
    }

    pub fn with_pixels<T>(&mut self, callback: |i32, i32, &[u8]|->T) -> T {
        logi("in with_pixels");
        let oldtarget = self.targetdata.get_current_texturetarget();
        let (x,y) = oldtarget.texture.dimensions;
        gl2::bind_framebuffer(gl2::FRAMEBUFFER, oldtarget.framebuffer);
        check_gl_error("read_pixels");
        // The only purpose of the shader copy is to flip the image from gl coords to bitmap coords.
        // it might be better to finagle the output copy matrix so the rest of the targets
        // can stay in bitmap coords?  Or have a dedicated target for this.
        let saveshader = Shader::new(None, Some(include_str!("../includes/shaders/noalpha_copy.fsh"))).unwrap();
        let newtarget = TextureTarget::new(x, y, gltexture::RGB);
        let matrix = [1f32,  0f32,  0f32,  0f32,
                      0f32, -1f32,  0f32,  0f32,
                      0f32,  0f32,  1f32,  0f32,
                      0f32,  1f32,  0f32,  1f32,];
        perform_copy(newtarget.framebuffer, &oldtarget.texture, &saveshader, matrix.as_slice());
        gl2::finish();
        let pixels = gl2::read_pixels(0, 0, x, y, gl2::RGBA, gl2::UNSIGNED_BYTE);
        check_gl_error("read_pixels");
        let result = callback(x, y, pixels.as_slice());
        logi("gl2::read_pixels()");
        result
    }

    // TODO: make an enum for these with a scala counterpart
    pub fn set_copy_shader(&mut self, shader: &'a CopyShader) -> () {
        logi("setting copy shader");
        self.paintstate.copyshader = Some(shader);
    }

    // these can also be null to unset the shader
    // TODO: document better from scala side
    pub fn set_anim_shader(&mut self, shader: &'a CopyShader) -> () {
        logi("setting anim shader");
        self.paintstate.animshader = Some(shader);
    }

    pub fn set_point_shader(&mut self, shader: &'a PointShader) -> () {
        logi("setting point shader");
        self.paintstate.pointshader = Some(shader);
    }

    pub fn set_interpolator(&mut self, interpolator: &'a LuaScript) -> () {
        logi("setting interpolator");
        self.paintstate.interpolator = Some(interpolator);
    }

    pub fn set_brush_texture(&mut self, texture: &'a Texture) {
        self.paintstate.brush = Some(texture);
    }

    pub fn add_layer(&mut self, layer: PaintLayer<'a>) -> () {
        logi("adding layer");
        let extra: i32 = (layer.pointidx as i32 + 1) - self.points.len() as i32;
        if extra > 0 {
            self.points.grow(extra as uint, Vec::new());
        }
        self.paintstate.layers.push(layer);
    }

    pub fn clear_layers(&mut self) {
        logi!("setting layer count to 0");
        self.paintstate.layers.clear();
        self.points.truncate(1);
    }

    #[inline]
    pub fn erase_layer(&mut self, layer: i32) {
        let target = match layer {
            0 => self.targetdata.get_current_texturetarget().framebuffer,
            _ => self.paintstate.layers[(layer - 1) as uint].target.framebuffer
        };
        gl2::bind_framebuffer(gl2::FRAMEBUFFER, target);
        gl2::clear_color(0f32, 0f32, 0f32, 0f32);
        gl2::clear(gl2::COLOR_BUFFER_BIT);
    }

    pub fn setup_graphics<'a>(w: i32, h: i32) -> GLInit<'a> {
        print_gl_string("Version", gl2::VERSION);
        print_gl_string("Vendor", gl2::VENDOR);
        print_gl_string("Renderer", gl2::RENDERER);
        print_gl_string("Extensions", gl2::EXTENSIONS);

        logi!("setupGraphics({},{})", w, h);
        let targets = [TextureTarget::new(w, h, gltexture::RGBA), TextureTarget::new(w, h, gltexture::RGBA)];
        let mut points: Vec<Vec<ShaderPaintPoint>> = Vec::new();
        points.push(Vec::new());
        let data = GLInit {
            dimensions: (w, h),
            targetdata: TargetData {
                targets: targets,
                current_target: 0,
            },
            points: points,
            paintstate: PaintState::new(),
        };

        gl2::viewport(0, 0, w, h);
        gl2::disable(gl2::DEPTH_TEST);
        gl2::blend_func(gl2::ONE, gl2::ONE_MINUS_SRC_ALPHA);

        data
    }

    pub fn draw_queued_points(&mut self, handler: &mut MotionEventConsumer, events: &'a mut Events<'a>, matrix: &matrix::Matrix) -> GLResult<()> {
        match (self.paintstate.pointshader, self.paintstate.copyshader, self.paintstate.brush) {
            (Some(point_shader), Some(copy_shader), Some(brush)) => {
                gl2::enable(gl2::BLEND);
                gl2::blend_func(gl2::ONE, gl2::ONE_MINUS_SRC_ALPHA);

                let (interp_error, should_copy) = if let Some(interpolator) = self.paintstate.interpolator {
                    let interp_error = unsafe {
                        do_interpolate_lua(interpolator, self.dimensions, &mut LuaCallbackType::new(self, events, handler))
                    };
                    let should_copy = handler.frame_done();
                    (interp_error, should_copy)
                } else {
                    (Ok(()), false)
                };

                let (target, source) = self.targetdata.get_texturetargets();
                let back_buffer = &source.texture;
                let drawvecs = self.points.as_mut_slice();
                let matrix = matrix.as_slice();
                let color = [1f32, 1f32, 0f32]; // TODO: brush color selection
                let baselayer = CompletedLayer {
                    copyshader: copy_shader,
                    pointshader: point_shader,
                    target: target,
                };
                draw_layer(baselayer, matrix, color, brush, back_buffer, drawvecs[0].as_slice());

                for layer in self.paintstate.layers.iter() {
                    let completed = layer.complete(copy_shader, point_shader);
                    let points = drawvecs[layer.pointidx as uint].as_slice();
                    draw_layer(completed, matrix, color, brush, back_buffer, points);

                    if should_copy {
                        let copymatrix = matrix::IDENTITY.as_slice();
                        perform_copy(target.framebuffer, &layer.target.texture, completed.copyshader, copymatrix);
                        gl2::bind_framebuffer(gl2::FRAMEBUFFER, layer.target.framebuffer);
                        gl2::clear_color(0f32, 0f32, 0f32, 0f32);
                        gl2::clear(gl2::COLOR_BUFFER_BIT);
                        logi!("copied brush layer down");
                    }
                }

                for drawvec in drawvecs.iter_mut() {
                    drawvec.clear();
                }

                interp_error
            },
            _ => { Ok(()) }
        }
    }

    pub fn clear_buffer(&mut self) {
        for target in self.targetdata.targets.iter() {
            gl2::bind_framebuffer(gl2::FRAMEBUFFER, target.framebuffer);
            gl2::clear_color(0f32, 0f32, 0f32, 0f32);
            gl2::clear(gl2::COLOR_BUFFER_BIT);
            check_gl_error("clear framebuffer");
        }
    }

    pub fn render_frame(&mut self) {
        match (self.paintstate.copyshader, self.paintstate.animshader) {
            (Some(copy_shader), Some(anim_shader)) => {
                self.targetdata.current_target = self.targetdata.current_target ^ 1;
                let copymatrix = matrix::IDENTITY.as_slice();
                gl2::disable(gl2::BLEND);
                let (target, source) = self.targetdata.get_texturetargets();
                perform_copy(target.framebuffer, &source.texture, anim_shader, copymatrix);
                perform_copy(0 as GLuint, &target.texture, copy_shader, copymatrix);
                gl2::enable(gl2::BLEND);
                for layer in self.paintstate.layers.iter() {
                    perform_copy(0 as GLuint, &layer.target.texture, layer.copyshader.unwrap_or(copy_shader), copymatrix);
                }
                eglinit::egl_swap();
            },
            (x, y) => {
                logi!("skipped frame! copyshader is {}, animshader is {}", x, y);
            }
        }
    }

    pub unsafe fn destroy(self) {
        gl2::finish();
    }
}

impl gltexture::ToPixelFormat for AndroidBitmapFormat {
    fn to_pixelformat(&self) -> GLResult<gltexture::PixelFormat> {
        match *self {
            ANDROID_BITMAP_FORMAT_RGBA_8888 => Ok(gltexture::RGBA),
            ANDROID_BITMAP_FORMAT_A_8 => Ok(gltexture::ALPHA),
            _ => Err("Unsupported texture format!".into_string()),
        }
    }
}

#[allow(dead_code, unused_must_use)]
fn test_all() {
    {
        //let mut data = GLInit::setup_graphics(0, 0);
        //let (mut consumer, producer) = create_motion_event_handler();
        //let copyshader = data.compile_copy_shader(None, None).unwrap();
        //let pointshader = data.compile_point_shader(None, None).unwrap();
        //let interpolator = data.compile_luascript(None).unwrap();
        //let brushpixels = [1u8, 0, 0, 1];
        //let brush = data.load_texture(1, 1, brushpixels, unsafe { mem::transmute(ANDROID_BITMAP_FORMAT_A_8) }).unwrap();

        //data.set_copy_shader(copyshader);
        //data.set_anim_shader(copyshader);
        //data.set_point_shader(pointshader);
        //data.set_interpolator(interpolator);
        //data.set_brush_texture(brush);
        //data.clear_layers();
        //data.add_layer(copyshader, pointshader, 0);
        //data.draw_image(1, 1, brushpixels);
        ////let point = ShaderPaintPoint { pos: ::point::Coordinate { x: 0f32, y: 0f32 }, time: 0f32, size: 0f32, speed: ::point::Coordinate { x: 0f32, y: 0f32 }, distance: 0f32, counter: 0f32 };
        ////data.points[0].push(point);
        
        //data.clear_buffer();
        //let result = data.draw_queued_points(&mut *consumer, &matrix::IDENTITY);
        //match result {
            //Err(err) => logi!("error drawing points: {}", err),
            //Ok(_)    => logi!("drew points successfully)"),
        //};
        //data.render_frame();
        //unsafe {
            //destroy_motion_event_handler(consumer, producer);
        //}
    }
}
