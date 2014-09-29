extern crate opengles;
use core::prelude::*;
use core::{mem,ptr};
use collections::vec::Vec;
use collections::{Mutable, MutableSeq};

use std::c_str::CString;

use log::logi;

use opengles::gl2;
use opengles::gl2::{GLuint, GLenum, GLubyte};

use glcommon::{Shader, check_gl_error};
use glpoint::{MotionEventConsumer, run_interpolators};
use point::ShaderPaintPoint;
use pointshader::PointShader;
use copyshader;
use copyshader::*;
use gltexture;
use gltexture::Texture;
use matrix;
use eglinit;
use drawevent::Events;
use glstore::DrawObjectIndex;
use luascript::LuaScript;

use alloc::boxed::Box;


static draw_indexes: [GLubyte, ..6] = [
    0, 1, 2,
    0, 2, 3
];

//#[deriving(FromPrimitive)]
#[repr(i32)]
#[allow(non_camel_case_types, dead_code)]
enum AndroidBitmapFormat {
    ANDROID_BITMAP_FORMAT_NONE      = 0,
    ANDROID_BITMAP_FORMAT_RGBA_8888 = 1,
    ANDROID_BITMAP_FORMAT_RGB_565   = 4,
    ANDROID_BITMAP_FORMAT_RGBA_4444 = 7,
    ANDROID_BITMAP_FORMAT_A_8       = 8,
}

struct TextureTarget {
    framebuffer: GLuint,
    texture: Texture,
}

impl TextureTarget {
    fn new(w: i32, h: i32, format: gltexture::PixelFormat) -> TextureTarget {
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
        logi!("deleted texturetarget: {} framebuffer {}", self.texture.dimensions, self.framebuffer);
    }
}

pub struct PaintLayer<'a> {
    copyshader: &'a CopyShader,
    pointshader: &'a PointShader,
    target: TextureTarget,
    points: &'a Vec<ShaderPaintPoint>,
}

impl<'a> PaintLayer<'a> {
    pub fn new(dimensions: (i32, i32), copyshader: &'a CopyShader, pointshader: &'a PointShader, points: &'a Vec<ShaderPaintPoint>) -> PaintLayer<'a> {
        let (w,h) = dimensions;
        PaintLayer {
            copyshader: copyshader,
            pointshader: pointshader,
            target: TextureTarget::new(w, h, gltexture::RGBA),
            points: points
        }
    }
}

/// struct for static storage of data that stays on rust side
pub struct Data<'a> {
    #[allow(dead_code)]
    dimensions: (i32, i32),
    events: Events<'a>,
    targetdata: TargetData,
    //baselayer: Option<PaintLayer<'a>>,
    layers: Vec<PaintLayer<'a>>,
    points: Vec<Vec<ShaderPaintPoint>>,
}

pub struct TargetData {
    targets: [TextureTarget, ..2],
    current_target: u8,
}

fn print_gl_string(name: &str, s: GLenum) {
    let glstr = gl2::get_string(s);
    logi!("GL {} = {}\n", name, glstr);
}

fn get_current_texturetarget<'a>(data: &'a TargetData) -> &'a TextureTarget {
    &data.targets[data.current_target as uint]
}

fn get_current_texturesource<'a> (data: &'a TargetData) -> &'a TextureTarget {
    &data.targets[(data.current_target ^ 1) as uint]
}

fn get_texturetargets<'a> (data: &'a TargetData) -> (&'a TextureTarget, &'a TextureTarget) {
    (get_current_texturetarget(data), get_current_texturesource(data))
}

fn perform_copy(dest_framebuffer: GLuint, source_texture: &Texture, shader: &CopyShader, matrix: &[f32]) -> () {
    gl2::bind_framebuffer(gl2::FRAMEBUFFER, dest_framebuffer);
    check_gl_error("bound framebuffer");
    shader.prep(source_texture, matrix);
    gl2::draw_elements(gl2::TRIANGLES, draw_indexes.len() as i32, gl2::UNSIGNED_BYTE, Some(draw_indexes.as_slice()));
    check_gl_error("drew elements");
}

#[no_mangle]
pub fn draw_image(data: *mut Data, w: i32, h: i32, pixels: *const u8) -> () {
    logi("drawing image...");
    let data = get_safe_data(data);
    let target = get_current_texturetarget(&data.targetdata);
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

    unsafe {
        pixels.as_ref().map(|x| ::core::slice::raw::buf_as_slice(x, (w*h*4) as uint, |pixelvec| {
            data.events.copyshader.map(|shader| {
                let intexture = Texture::with_image(w, h, Some(pixelvec), gltexture::RGBA);
                check_gl_error("creating texture");
                perform_copy(target.framebuffer, &intexture, shader, matrix.as_slice());
            });
        }));
    }
}

#[no_mangle]
pub unsafe fn with_pixels(data: *mut Data, callback: unsafe extern "C" fn(i32, i32, *const u8, *mut ())-> *mut (), env: *mut ()) -> *mut () {
    logi("in with_pixels");
    let data = get_safe_data(data);
    let oldtarget = get_current_texturetarget(&data.targetdata);
    let (x,y) = oldtarget.texture.dimensions;
    let saveshader = Shader::new(None, Some(copyshader::noalpha_fragment_shader));
    saveshader.map(|shader| {
        let newtarget = TextureTarget::new(x, y, gltexture::RGB);
        let matrix = [1f32,  0f32,  0f32,  0f32,
                      0f32, -1f32,  0f32,  0f32,
                      0f32,  0f32,  1f32,  0f32,
                      0f32,  1f32,  0f32,  1f32,];
        perform_copy(newtarget.framebuffer, &oldtarget.texture, &shader, matrix.as_slice());
        gl2::finish();
        let pixels = gl2::read_pixels(0, 0, x, y, gl2::RGBA, gl2::UNSIGNED_BYTE);
        check_gl_error("read_pixels");
        logi("gl2::read_pixels()");
        let pixptr = pixels.as_ptr();
        logi!("calling callback");
        let result = callback(x, y, pixptr, env);
        logi!("returning pixels: {}", pixptr);
        result
    }).unwrap_or(ptr::null_mut())
}

unsafe fn with_cstr_as_str<T>(ptr: *const i8, callback: |Option<&str>|->T)->T {
    let cstr = ptr.as_ref().map_or(None, |b| Some(CString::new(b, false)));
    let vecstr = cstr.as_ref().and_then(|b| b.as_str());
    callback(vecstr)
}

unsafe fn compile_shader<T>(vec: *const i8, frag: *const i8, 
                 callback: |Option<&str>, Option<&str>| -> T) -> T {
    // note that this will use the default shader in case of non-utf8 chars
    // also, must be separate lines b/c ownership
    with_cstr_as_str(vec, |vecstr| with_cstr_as_str(frag, |fragstr| {
        callback(vecstr, fragstr)
    }))
}

#[no_mangle]
pub unsafe fn compile_copy_shader(data: *mut Data, vert: *const i8, frag: *const i8) -> DrawObjectIndex<CopyShader> {
    let shader = compile_shader(vert, frag, |v,f|get_safe_data(data).events.load_copyshader(v,f));
    shader.unwrap_or(mem::transmute(-1i))
}

#[no_mangle]
pub unsafe fn compile_point_shader(data: *mut Data, vert: *const i8, frag: *const i8) -> DrawObjectIndex<PointShader> {
    let shader = compile_shader(vert, frag, |v,f|get_safe_data(data).events.load_pointshader(v,f));
    shader.unwrap_or(mem::transmute(-1i))
}

#[no_mangle]
pub unsafe fn compile_luascript(data: *mut Data, luachars: *const i8) -> DrawObjectIndex<LuaScript> {
    let script = with_cstr_as_str(luachars, |luastr| {
        get_safe_data(data).events.load_interpolator(luastr)
    });
    script.unwrap_or(mem::transmute(-1i))
}

// TODO: make an enum for these with a scala counterpart
#[no_mangle]
pub unsafe fn set_copy_shader(data: *mut Data, shader: DrawObjectIndex<CopyShader>) -> () {
    logi("setting copy shader");
    get_safe_data(data).events.use_copyshader(shader);
}

// these can also be null to unset the shader
// TODO: document better from scala side
#[no_mangle]
pub unsafe fn set_anim_shader(data: *mut Data, shader: DrawObjectIndex<CopyShader>) -> () {
    logi("setting anim shader");
    get_safe_data(data).events.use_animshader(shader);
}

#[no_mangle]
pub unsafe fn set_point_shader(data: *mut Data, shader: DrawObjectIndex<PointShader>) -> () {
    logi("setting point shader");
    get_safe_data(data).events.use_pointshader(shader);
}

#[no_mangle]
pub unsafe fn set_interpolator(data: *mut Data, interpolator: DrawObjectIndex<LuaScript>) -> () {
    logi("setting interpolator");
    get_safe_data(data).events.use_interpolator(interpolator);
}

#[no_mangle]
pub unsafe fn add_layer(data: *mut Data, copyshader: DrawObjectIndex<CopyShader>, pointshader: DrawObjectIndex<PointShader>, pointidx: i32) -> () {
    logi("adding layer");
    let data = get_safe_data(data);
    let (copy, point) = data.events.add_layer(copyshader, pointshader, pointidx);
    let extra: i32 = pointidx as i32 - data.points.len() as i32;
    if extra > 0 {
        data.points.grow(extra as uint, Vec::new());
    }
    data.layers.push(PaintLayer::new(data.dimensions, copy, point, &data.points[pointidx as uint]));
}

#[no_mangle]
pub unsafe fn set_layer_count(data: *mut Data, count: i32) {
    logi!("setting layer count to {}", count);
    let data = get_safe_data(data);
    data.layers.truncate(count as uint);
    // FIXME shrink points array to fit
}

//#[no_mangle]
//pub unsafe fn set_separate_brushlayer(data: *mut Data, separate_layer: bool) -> () {
    //let data = get_safe_data(data);
    //let currentlayers = data.layers.len();
    //match (data.events.pointshader, data.events.copyshader)
    //match (data.brushlayer.as_ref(), separate_layer) {
        //(_, false) => {
            //data.brushlayer = None;
        //},
        //(None, true) => {
            //let (w,h) = data.dimensions;
            //data.brushlayer = Some(TextureTarget::new(w, h, gltexture::RGBA));
        //},
        //_ => { },
    //};
//}

#[no_mangle]
pub extern fn setup_graphics<'a>(w: i32, h: i32) -> *mut Data<'a> {
    print_gl_string("Version", gl2::VERSION);
    print_gl_string("Vendor", gl2::VENDOR);
    print_gl_string("Renderer", gl2::RENDERER);
    print_gl_string("Extensions", gl2::EXTENSIONS);

    logi!("setupGraphics({},{})", w, h);
    let targets = [TextureTarget::new(w, h, gltexture::RGBA), TextureTarget::new(w, h, gltexture::RGBA)];
    let mut points: Vec<Vec<ShaderPaintPoint>> = Vec::new();
    points.push(Vec::new());
    let data = box Data {
        dimensions: (w, h),
        events: Events::new(),
        targetdata: TargetData {
            targets: targets,
            current_target: 0,
        },
        layers: Vec::new(),
        points: points,
    };

    gl2::viewport(0, 0, w, h);
    gl2::disable(gl2::DEPTH_TEST);
    gl2::blend_func(gl2::ONE, gl2::ONE_MINUS_SRC_ALPHA);
    unsafe {
        let dataptr: *mut Data = mem::transmute(data);
        dataptr
    }
}

fn get_safe_data(data: *mut Data) -> &mut Data {
    unsafe { &mut *data }
}

#[no_mangle]
pub extern fn draw_queued_points(data: *mut Data, handler: *mut MotionEventConsumer, matrix: *mut f32) {
    let data = get_safe_data(data);
    match (data.events.pointshader, data.events.brush) {
        (Some(point_shader), Some(brush)) => {
            gl2::enable(gl2::BLEND);
            gl2::blend_func(gl2::ONE, gl2::ONE_MINUS_SRC_ALPHA);
            let (target, source) = get_texturetargets(&data.targetdata);
            // TODO: brush color selection


            let safe_matrix: &matrix::Matrix = unsafe { mem::transmute(matrix) };
            let drawvecs = data.points.as_mut_slice();

            for drawvec in drawvecs.iter_mut() {
                drawvec.clear();
            }
            let should_copy = run_interpolators(data.dimensions, handler, &mut data.events, drawvecs);
            let color = [1f32, 1f32, 0f32];
            let back_buffer = &source.texture;

            for layer in data.layers.iter() {
                if layer.points.len() > 0 {
                    gl2::bind_framebuffer(gl2::FRAMEBUFFER, layer.target.framebuffer);
                    layer.pointshader.prep(safe_matrix.as_slice(), layer.points.as_slice(), color, brush, back_buffer);
                    gl2::draw_arrays(gl2::POINTS, 0, layer.points.len() as i32);
                    check_gl_error("draw_arrays");
                }
            }

            let basepointvec = drawvecs[0].as_slice();
            gl2::bind_framebuffer(gl2::FRAMEBUFFER, target.framebuffer);
            point_shader.prep(safe_matrix.as_slice(), basepointvec, color, brush, back_buffer);
            gl2::draw_arrays(gl2::POINTS, 0, basepointvec.len() as i32);

            let copymatrix = matrix::identity.as_slice();
            if should_copy {
                for layer in data.layers.iter() {
                    perform_copy(target.framebuffer, &layer.target.texture, layer.copyshader, copymatrix);
                    gl2::bind_framebuffer(gl2::FRAMEBUFFER, layer.target.framebuffer);
                    gl2::clear_color(0f32, 0f32, 0f32, 0f32);
                    gl2::clear(gl2::COLOR_BUFFER_BIT);
                    logi!("copied brush layer down");
                }
            }
        },
        _ => { }
    }
}

#[no_mangle]
pub extern fn load_texture(data: *mut Data, w: i32, h: i32, a_pixels: *const u8, format: i32) -> i32 {
    let data = get_safe_data(data);
    let formatenum: AndroidBitmapFormat = unsafe { mem::transmute(format) };
    let format_and_size = match formatenum {
        ANDROID_BITMAP_FORMAT_RGBA_8888 => Some((gltexture::RGBA, 4)),
        ANDROID_BITMAP_FORMAT_A_8 => Some((gltexture::ALPHA, 1)),
        _ => None,
    };
    format_and_size.and_then(|(texformat, size)| {
        logi!("setting brush texture for {:x}", a_pixels as uint);
        let pixelopt = unsafe { a_pixels.as_ref() };
        unsafe {
            pixelopt.map(|x| ::std::slice::raw::buf_as_slice(x, (w*h*size) as uint, |x| {
                mem::transmute(data.events.load_brush(w, h, x, texformat))
            }))
        }
    }).unwrap_or(-1)
}

#[no_mangle]
pub extern fn set_brush_texture(data: *mut Data, texture: i32) {
    get_safe_data(data).events.use_brush(unsafe { mem::transmute(texture) });
}

#[no_mangle]
pub extern fn clear_buffer(data: *mut Data) {
    let data = get_safe_data(data);
    for target in data.targetdata.targets.iter() {
        gl2::bind_framebuffer(gl2::FRAMEBUFFER, target.framebuffer);
        gl2::clear_color(0f32, 0f32, 0f32, 0f32);
        gl2::clear(gl2::COLOR_BUFFER_BIT);
        check_gl_error("clear framebuffer");
        data.events.clear();
    }
}

#[no_mangle]
pub extern fn render_frame(data: *mut Data) {
    let data = get_safe_data(data);
    match (data.events.copyshader, data.events.animshader) {
        (Some(copy_shader), Some(anim_shader)) => {
            data.events.pushframe();
            data.targetdata.current_target = data.targetdata.current_target ^ 1;
            let copymatrix = matrix::identity.as_slice();
            gl2::disable(gl2::BLEND);
            let (target, source) = get_texturetargets(&data.targetdata);
            perform_copy(target.framebuffer, &source.texture, anim_shader, copymatrix);
            perform_copy(0 as GLuint, &target.texture, copy_shader, copymatrix);
            gl2::enable(gl2::BLEND);
            for layer in data.layers.iter() {
                perform_copy(0 as GLuint, &layer.target.texture, layer.copyshader, copymatrix);
            }
            eglinit::egl_swap();
        },
        (x, y) => {
            logi!("skipped frame! copyshader is {}, animshader is {}", x, y);
        }
    }
}

#[no_mangle]
pub unsafe extern fn deinit_gl(data: *mut Data) {
    let data: Box<Data> = mem::transmute(data);
    mem::drop(data);
    gl2::finish();
}
