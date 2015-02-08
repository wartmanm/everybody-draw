use egl::egl::*;
use core::prelude::*;
use libc::{c_void, c_uint};
use core::mem;

static DEFAULT_EGL_CONFIG: [u32; 15] = [
    EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
    EGL_RED_SIZE, 8,
    EGL_GREEN_SIZE, 8,
    EGL_BLUE_SIZE, 8,
    EGL_ALPHA_SIZE, 8,
    EGL_DEPTH_SIZE, 0,
    EGL_STENCIL_SIZE, 0,
    EGL_NONE
];

static DEFAULT_CONTEXT_ATTRIBS: [u32; 3] = [
    EGL_CONTEXT_CLIENT_VERSION, 2,
    EGL_NONE
];

static NO_ATTRIBS: [u32; 1] = [ EGL_NONE ];

struct EGLStatus {
    display: EGLDisplay,
    context: EGLContext,
    surface: EGLSurface,
}

static mut data: Option<EGLStatus> = None;

fn get_config() -> *const i32 {
    unsafe { mem::transmute(DEFAULT_EGL_CONFIG.as_slice().as_ptr()) }
}

fn get_context_attribs() -> *const i32 {
    unsafe { mem::transmute(DEFAULT_CONTEXT_ATTRIBS.as_slice().as_ptr()) }
}

fn get_no_attribs() -> *const i32 {
    unsafe { mem::transmute(NO_ATTRIBS.as_slice().as_ptr()) }
}

fn choose_egl_config(display: EGLDisplay) -> Option<EGLConfig> {
    let mut config_count = 0;
    let mut config: EGLConfig = 0 as EGLConfig;
    let configattrs = get_config();
    debug_logi!("choosing config... attrs = {}", configattrs as uint);
    if EGL_TRUE != ChooseConfig(display, get_config(), &mut config, 1, &mut config_count) {
        loge!("eglChooseConfig returned false :(");
        None
    } else if config_count == 0 {
        loge!("no matching configs found :(");
        None
    } else {
        debug_logi!("got {} configs", config_count);
        Some(config)
    }
}

fn get_display(display: EGLNativeDisplayType) -> Option<EGLDisplay> {
    // with the correct EGL_Display etc types,
    // this errors with "can't cast this type" on the first line of lib.rs
    match GetDisplay(display) as c_uint {
        EGL_NO_DISPLAY => None,
        x => Some(x as EGLDisplay),
    }
    //None
}

#[no_mangle]
pub fn egl_init(window: *mut c_void) -> () {
    unsafe { data = init_context(window) };
}

fn init_context(surface_texture: *mut c_void) -> Option<EGLStatus> {
    let displayopt = get_display(EGL_DEFAULT_DISPLAY as EGLNativeDisplayType);
    if displayopt.is_none() {
        loge!("failed to get display :(\n");
        return None;
    }
    let display = displayopt.unwrap();
    debug_logi!("got display: {:?}", display);
    let (mut vermajor, mut verminor) = (0, 0);
    if Initialize(display, &mut vermajor, &mut verminor) != EGL_TRUE {
        loge!("failed to initialize display :(");
        return None;
    }
    debug_logi!("initialized display!");
    loge!("egl {}.{}\n", vermajor, verminor);
    loge!("extensions: {}\n", QueryString(display, EGL_EXTENSIONS));
    loge!("vendor: {}\n", QueryString(display, EGL_VENDOR));

    let configopt = choose_egl_config(display);
    if configopt.is_none() {
        loge!("failed to get config :(");
        return None;
    }
    let config = configopt.unwrap();
    //logi!("got config: 0x{:x}", config);
    debug_logi!("creating context...");
    let context = CreateContext(display, config, EGL_NO_CONTEXT as *mut c_void, get_context_attribs());
    debug_logi!("got context: 0x{:x}", context as uint);
    debug_logi!("creating window surface...");
    let surface = CreateWindowSurface(display, config, surface_texture, get_no_attribs());
    if surface == EGL_NO_SURFACE as *mut c_void {
        debug_logi!("getting error...");
        let error = GetError();
        match error {
            EGL_BAD_NATIVE_WINDOW => {
                loge!("createwindowsurface returned EGL_BAD_NATIVE_WINDOW :(");
            },
            x => {
                loge!("createwindowsurface failed: {}", x);
            },
        }
        return None;
    }
    debug_logi!("got surface: 0x{:x}", surface as uint);

    if MakeCurrent(display, surface, surface, context) != EGL_TRUE {
        loge!("eglMakeCurrent failed");
        return None
    }
    debug_logi!("made egl surface current!");
    Some(EGLStatus { display: display, context: context, surface: surface })
}

#[no_mangle]
pub fn egl_swap() -> () {
    unsafe {
        match data {
            Some(EGLStatus { display, context: _, surface }) => {
                if SwapBuffers(display, surface) != EGL_TRUE {
                    loge!("failed to swap buffers??");
                }
            },
            None => { },
        }
    }
}
        

#[no_mangle]
pub fn egl_finish() -> () {
    unsafe {
        match data {
            Some(EGLStatus { display, context, surface }) => {
                debug_logi!("running finish_egl");
                MakeCurrent(display, EGL_NO_SURFACE as *mut c_void, EGL_NO_SURFACE as *mut c_void, EGL_NO_CONTEXT as *mut c_void);
                debug_logi!("detached from egl");
                DestroySurface(display, surface);
                debug_logi!("destroyed surface");
                DestroyContext(display, context);
                debug_logi!("destroyed context");
                Terminate(display);
                logi!("finished egl");
            },
            None => { },
        }
    }
}

