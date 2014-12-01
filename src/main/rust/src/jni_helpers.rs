use jni::*;
use core::mem;

pub trait ToJValue {
    fn as_jvalue(&mut self) -> jvalue;
}
impl ToJValue for jboolean { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
}
impl ToJValue for jbyte { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
}
impl ToJValue for jchar { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
}
impl ToJValue for jshort { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
}
impl ToJValue for jint { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
}
impl ToJValue for jlong { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute(*self) }
    }
}
impl ToJValue for jfloat { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
}
impl ToJValue for jdouble { 
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute(*self) }
    }
}
impl ToJValue for jobject { 
    #[cfg(target_word_size = "32")]
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute((*self, mem::uninitialized::<i32>())) }
    }
    #[cfg(target_word_size = "64")]
    #[inline(always)]
    fn as_jvalue(&mut self) -> jvalue {
        unsafe { mem::transmute(*self) }
    }
}
