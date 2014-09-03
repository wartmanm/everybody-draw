extern crate opengles;
use core::prelude::*;
use core::{mem,ptr};

use std::c_str::CString;

use log::logi;

use opengles::gl2;
use opengles::gl2::{GLuint, GLenum, GLubyte};

use glcommon::{Shader, check_gl_error};
use glpoint::{MotionEventConsumer, draw_path};
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


static drawIndexes: [GLubyte, ..6] = [
    0, 1, 2,
    0, 2, 3
];

//#[deriving(FromPrimitive)]
#[repr(i32)]
#[allow(non_camel_case_types)]
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

/// struct for static storage of data that stays on rust side
pub struct Data<'a> {
    #[allow(dead_code)]
    dimensions: (i32, i32),
    events: Events<'a>,
    targets: [TextureTarget, ..2],
    current_target: uint,
    brushlayer: Option<TextureTarget>,
}

fn print_gl_string(name: &str, s: GLenum) {
    let glstr = gl2::get_string(s);
    logi!("GL {} = {}\n", name, glstr);
}

fn get_current_texturetarget<'a>(data: &'a Data) -> &'a TextureTarget {
    &data.targets[data.current_target]
}

fn get_current_texturesource<'a> (data: &'a Data) -> &'a TextureTarget {
    &data.targets[data.current_target ^ 1]
}

fn get_texturetargets<'a> (data: &'a Data) -> (&'a TextureTarget, &'a TextureTarget) {
    (get_current_texturetarget(data), get_current_texturesource(data))
}

fn perform_copy(destFramebuffer: GLuint, sourceTexture: &Texture, shader: &CopyShader, matrix: &[f32]) -> () {
    gl2::bind_framebuffer(gl2::FRAMEBUFFER, destFramebuffer);
    check_gl_error("bound framebuffer");
    shader.prep(sourceTexture, matrix);
    gl2::draw_elements(gl2::TRIANGLES, drawIndexes.len() as i32, gl2::UNSIGNED_BYTE, Some(drawIndexes.as_slice()));
    check_gl_error("drew elements");
}

#[no_mangle]
pub fn draw_image(data: *mut Data, w: i32, h: i32, pixels: *const u8) -> () {
    logi("drawing image...");
    let data = get_safe_data(data);
    let target = get_current_texturetarget(data);
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
        pixels.to_option().map(|x| ::core::slice::raw::buf_as_slice(x, (w*h*4) as uint, |pixelvec| {
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
    let oldtarget = get_current_texturetarget(data);
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
    }).unwrap_or(ptr::mut_null())
}

unsafe fn with_cstr_as_str<T>(ptr: *const i8, callback: |Option<&str>|->T)->T {
    let cstr = ptr.to_option().map_or(None, |b| Some(CString::new(b, false)));
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
pub unsafe fn set_separate_brushlayer(data: *mut Data, separate_layer: bool) -> () {
    let data = get_safe_data(data);
    match (data.brushlayer.as_ref(), separate_layer) {
        (_, false) => {
            data.brushlayer = None;
        },
        (None, true) => {
            let (w,h) = data.dimensions;
            data.brushlayer = Some(TextureTarget::new(w, h, gltexture::RGBA));
        },
        _ => { },
    };
}

#[no_mangle]
pub extern fn setup_graphics<'a>(w: i32, h: i32) -> *mut Data<'a> {
    print_gl_string("Version", gl2::VERSION);
    print_gl_string("Vendor", gl2::VENDOR);
    print_gl_string("Renderer", gl2::RENDERER);
    print_gl_string("Extensions", gl2::EXTENSIONS);

    logi!("setupGraphics({},{})", w, h);
    let targets = [TextureTarget::new(w, h, gltexture::RGBA), TextureTarget::new(w, h, gltexture::RGBA)];
    let data = box Data {
        dimensions: (w, h),
        events: Events::new(),
        targets: targets,
        current_target: 0,
        brushlayer: None,
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
    match (data.events.pointshader, data.events.brush, data.events.interpolator) {
        (Some(point_shader), Some(brush), Some(interpolator)) => {
            gl2::enable(gl2::BLEND);
            gl2::blend_func(gl2::ONE, gl2::ONE_MINUS_SRC_ALPHA);
            let (target, source) = get_texturetargets(data);
            // TODO: brush color selection
            let brushtarget = data.brushlayer.as_ref().unwrap_or(target);
            let should_copy = draw_path(handler, brushtarget.framebuffer, point_shader, interpolator,
                                        matrix, [1f32, 1f32, 0f32], brush, &source.texture);
            if should_copy && data.brushlayer.is_some() {
                match data.events.copyshader {
                    Some(copy_shader) => {
                        let copymatrix = matrix::identity.as_slice();
                        perform_copy(target.framebuffer, &brushtarget.texture, copy_shader, copymatrix);
                        gl2::bind_framebuffer(gl2::FRAMEBUFFER, brushtarget.framebuffer);
                        gl2::clear_color(0f32, 0f32, 0f32, 0f32);
                        gl2::clear(gl2::COLOR_BUFFER_BIT);
                        logi!("copied brush layer down");
                    },
                    None => {
                        logi!("not copying brush layer");
                    }
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
        let pixelopt = unsafe { a_pixels.to_option() };
        // pixelvec has lifetime of a_pixels, not x
        // there must be some way around this
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
    for target in data.targets.iter() {
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
            data.current_target = data.current_target ^ 1;
            let copymatrix = matrix::identity.as_slice();
            gl2::disable(gl2::BLEND);
            let (target, source) = get_texturetargets(data);
            perform_copy(target.framebuffer, &source.texture, anim_shader, copymatrix);
            perform_copy(0 as GLuint, &target.texture, copy_shader, copymatrix);
            match data.brushlayer.as_ref() {
                Some(ref brushtarget) => {
                    gl2::enable(gl2::BLEND);
                    perform_copy(0 as GLuint, &brushtarget.texture, copy_shader, copymatrix);
                },
                None => { },
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
