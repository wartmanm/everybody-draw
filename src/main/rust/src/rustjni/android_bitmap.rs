use core::prelude::*;
use core::raw;
use core::{ptr, mem};
use core::borrow::IntoCow;
use libc::c_void;
use jni::{jobject, jclass, jmethodID, JNIEnv};
use jni_constants::{JNI_TRUE, JNI_FALSE};
use android::bitmap::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels, AndroidBitmapInfo};
use android::bitmap::{ANDROID_BITMAP_FORMAT_NONE, ANDROID_BITMAP_FORMAT_RGBA_8888, ANDROID_BITMAP_FORMAT_RGB_565, ANDROID_BITMAP_FORMAT_RGBA_4444, ANDROID_BITMAP_FORMAT_A_8};
use gltexture;
use gltexture::PixelFormat;
use glcommon::GLResult;
use core::fmt;
use core::fmt::Debug;

static mut BITMAP_CLASS: jclass = 0 as jclass;
static mut CONFIG_ARGB_8888: jobject = 0 as jobject;
static mut CREATE_BITMAP: jmethodID = 0 as jmethodID;
static mut SET_PREMULTIPLIED: Option<jmethodID> = None;

pub struct AndroidBitmap {
    env: *mut JNIEnv,
    pub obj: jobject,
    pixels: *mut u8,
    pub info: AndroidBitmapInfo,
}

#[repr(i32)]
#[derive(Copy)]
#[allow(non_camel_case_types, dead_code)]
pub struct AndroidBitmapFormat {
    value: i32,
}

impl Debug for AndroidBitmapFormat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.value as u32 {
            ANDROID_BITMAP_FORMAT_NONE      => write!(fmt, "ANDROID_BITMAP_FORMAT_NONE"),
            ANDROID_BITMAP_FORMAT_RGBA_8888 => write!(fmt, "ANDROID_BITMAP_FORMAT_RGBA_8888"),
            ANDROID_BITMAP_FORMAT_RGB_565   => write!(fmt, "ANDROID_BITMAP_FORMAT_RGB_565"),
            ANDROID_BITMAP_FORMAT_RGBA_4444 => write!(fmt, "ANDROID_BITMAP_FORMAT_RGBA_4444"),
            ANDROID_BITMAP_FORMAT_A_8       => write!(fmt, "ANDROID_BITMAP_FORMAT_A_8"),
            other                           => write!(fmt, "Unknown bitmap format: {}", other),
        }
    }
}

impl gltexture::ToPixelFormat for AndroidBitmapFormat {
    fn to_pixelformat(&self) -> GLResult<PixelFormat> {
        match self.value as u32 {
            ANDROID_BITMAP_FORMAT_RGBA_8888 => Ok(PixelFormat::RGBA),
            ANDROID_BITMAP_FORMAT_A_8 => Ok(PixelFormat::ALPHA),
            _ => Err(format!("Unsupported texture format: {:?}!", self).into_cow()),
        }
    }
}

impl AndroidBitmap {
    pub unsafe fn from_jobject(env: *mut JNIEnv, bitmap: jobject) -> AndroidBitmap {
        let mut pixels: *mut c_void = ptr::null_mut();
        AndroidBitmap_lockPixels(env, bitmap, &mut pixels);
        debug_logi!("locked pixels in {:?}", pixels);
        let mut result = AndroidBitmap { env: env, obj: bitmap, pixels: pixels as *mut u8, info: mem::zeroed() };
        AndroidBitmap_getInfo(env, bitmap, &mut result.info);
        result
    }

    pub unsafe fn new(env: *mut JNIEnv, w: i32, h: i32) -> AndroidBitmap {
        let bitmap = ((**env).CallStaticObjectMethod)(env, BITMAP_CLASS, CREATE_BITMAP, w, h, CONFIG_ARGB_8888);
        debug_logi!("created bitmap");
        AndroidBitmap::from_jobject(env, bitmap)
    }
    
    unsafe fn as_slice_unsafe(&self) -> GLResult<&mut [u8]> {
        let pixelsize = match self.info.format as u32 {
            ANDROID_BITMAP_FORMAT_RGBA_8888 => 4,
            ANDROID_BITMAP_FORMAT_A_8 => 1,
            other => {
                return Err(format!("bitmap format {} not implemented!", other).into_cow());
            },
        };
        let pixelvec = raw::Slice { data: self.pixels as *const u8, len: (self.info.width * self.info.height * pixelsize) as usize };
        Ok(mem::transmute(pixelvec))
    }

    pub unsafe fn as_mut_slice(&mut self) -> GLResult<&mut [u8]> {
        self.as_slice_unsafe()
    }

    pub unsafe fn as_slice(&self) -> GLResult<&[u8]> {
        match self.as_slice_unsafe() {
            Ok(ok) => Ok(&*ok),
            Err(err) => Err(err),
        }
    }

    pub unsafe fn set_premultiplied(&mut self, premultiplied: bool) -> bool {
        if let Some(set_premultiplied) = SET_PREMULTIPLIED {
            let pm = if premultiplied { JNI_TRUE } else { JNI_FALSE };
            ((**self.env).CallVoidMethod)(self.env, self.obj, set_premultiplied, pm);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn get_format(&self) -> AndroidBitmapFormat {
        AndroidBitmapFormat { value: self.info.format }
    }
}

impl Drop for AndroidBitmap {
    fn drop(&mut self) {
        unsafe {
            AndroidBitmap_unlockPixels(self.env, self.obj);
        }
        debug_logi!("unlocked pixels");
    }
}

pub unsafe fn init(env: *mut JNIEnv) {
    let bitmapclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap"));
    let configclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap$Config"));
    let argbfield = ((**env).GetStaticFieldID)(env, configclass, cstr!("ARGB_8888"), cstr!("Landroid/graphics/Bitmap$Config;"));
    let argb = ((**env).GetStaticObjectField)(env, configclass, argbfield);
    let createbitmap = ((**env).GetStaticMethodID)(env, bitmapclass, cstr!("createBitmap"), cstr!("(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;"));
    BITMAP_CLASS = ((**env).NewGlobalRef)(env, bitmapclass);
    CONFIG_ARGB_8888 = ((**env).NewGlobalRef)(env, argb);
    CREATE_BITMAP = createbitmap;

    let premult = ((**env).GetMethodID)(env, bitmapclass, cstr!("setPremultiplied"), cstr!("(Z)V"));
    SET_PREMULTIPLIED = if premult == ptr::null_mut() {
        ((**env).ExceptionClear)(env);
         None
    } else {
        Some(premult)
    }
}

pub unsafe fn destroy(env: *mut JNIEnv) {
    ((**env).DeleteGlobalRef)(env, BITMAP_CLASS);
    ((**env).DeleteGlobalRef)(env, CONFIG_ARGB_8888);
}
