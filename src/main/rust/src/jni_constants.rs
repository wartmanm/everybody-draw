#![allow(unused_attribute)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_uppercase_statics)]

pub static JNI_FALSE: i32 = 0;
pub static JNI_TRUE:  i32 = 1;

pub static JNI_VERSION_1_1: i32 = 0x00010001;
pub static JNI_VERSION_1_2: i32 = 0x00010002;
pub static JNI_VERSION_1_4: i32 = 0x00010004;
pub static JNI_VERSION_1_6: i32 = 0x00010006;

pub static JNI_OK:        i32 = (0);         /* no error */
pub static JNI_ERR:       i32 = (-1);        /* generic error */
pub static JNI_EDETACHED: i32 = (-2);        /* thread detached from the VM */
pub static JNI_EVERSION:  i32 = (-3);        /* JNI version error */

pub static JNI_COMMIT:    i32 = 1;           /* copy content, do not free buffer */
pub static JNI_ABORT:     i32 = 2;           /* free buffer w/o copying back */

