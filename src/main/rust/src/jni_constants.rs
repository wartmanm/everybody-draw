#![allow(unused_attributes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const JNI_FALSE: i32 = 0;
pub const JNI_TRUE:  i32 = 1;

pub const JNI_VERSION_1_1: i32 = 0x00010001;
pub const JNI_VERSION_1_2: i32 = 0x00010002;
pub const JNI_VERSION_1_4: i32 = 0x00010004;
pub const JNI_VERSION_1_6: i32 = 0x00010006;

pub const JNI_OK:        i32 = (0);         /* no error */
pub const JNI_ERR:       i32 = (-1);        /* generic error */
pub const JNI_EDETACHED: i32 = (-2);        /* thread detached from the VM */
pub const JNI_EVERSION:  i32 = (-3);        /* JNI version error */

pub const JNI_COMMIT:    i32 = 1;           /* copy content, do not free buffer */
pub const JNI_ABORT:     i32 = 2;           /* free buffer w/o copying back */

