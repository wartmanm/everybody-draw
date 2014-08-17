use alloc::boxed::Box;
use core::mem;

/// at the time of writing, types with Drop are forbidden for static variables, understandably
/// however, there's no way around it
/// the best I could come up with is to store an (undroppable) unsafe pointer instead
pub struct DropFree<T>(pub *mut T);

impl<T> DropFree<T> {
    pub unsafe fn new(value: T) -> DropFree<T> {
        DropFree(mem::transmute(box value))
    }
    pub unsafe fn get_mut(&mut self) -> &mut T {
        let DropFree(value) = *self;
        mem::transmute(value)
    }
    pub unsafe fn get(&self) -> &T {
        let DropFree(value) = *self;
        mem::transmute(value)
    }
    pub unsafe fn destroy(&mut self) {
        let DropFree(value) = *self;
        let boxed: Box<T> = mem::transmute(value);
        mem::drop(boxed);
    }
}
        


