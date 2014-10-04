#![allow(unused_variable, dead_code)]

static NEWMASK: u8 = 0x01;
static OLDMASK: u8 = 0x02;

pub static STARTING: ActiveState = ActiveState(NEWMASK);
pub static STOPPING: ActiveState  = ActiveState(OLDMASK);
pub static CONTINUING: ActiveState  = ActiveState(NEWMASK | OLDMASK);
pub static INACTIVE: ActiveState = ActiveState(0);
#[deriving(Eq, PartialEq)]
pub struct ActiveState(u8);

impl ActiveState {
    #[inline]
    pub fn push(self, newstate: bool) -> ActiveState {
        let ActiveState(state) = self;
        ActiveState(((state << 1) & OLDMASK) | newstate as u8)
    }
}

