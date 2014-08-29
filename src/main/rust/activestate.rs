#![allow(unused_variable, dead_code)]

static newmask: u8 = 0x01;
static oldmask: u8 = 0x02;

pub static starting: ActiveState = ActiveState(newmask);
pub static stopping: ActiveState  = ActiveState(oldmask);
pub static continuing: ActiveState  = ActiveState(newmask | oldmask);
pub static inactive: ActiveState = ActiveState(0);
#[deriving(Eq, PartialEq)]
pub struct ActiveState(u8);

impl ActiveState {
    #[inline]
    pub fn push(self, newstate: bool) -> ActiveState {
        let ActiveState(state) = self;
        ActiveState(((state << 1) & oldmask) | newstate as u8)
    }
}

