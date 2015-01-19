#![allow(unused_variables, dead_code)]
use core::prelude::*;

const NEWMASK: u8 = 0x01;
const OLDMASK: u8 = 0x02;

pub const STARTING: ActiveState = ActiveState(NEWMASK);
pub const STOPPING: ActiveState  = ActiveState(OLDMASK);
pub const CONTINUING: ActiveState  = ActiveState(NEWMASK | OLDMASK);
pub const INACTIVE: ActiveState = ActiveState(0);
#[derive(Eq, PartialEq, Copy)]
pub struct ActiveState(u8);

impl ActiveState {
    #[inline]
    pub fn push(self, newstate: bool) -> ActiveState {
        let ActiveState(state) = self;
        ActiveState(((state << 1) & OLDMASK) | newstate as u8)
    }
    //#[inline]
    //pub fn is_active(self) -> bool {
        //(self & NEWMASK) != 0
    //}
}
