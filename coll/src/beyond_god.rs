use crate::impl_unpack;


////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Default, Clone, Copy)]
#[repr(transparent)]
pub struct UnsafeSendSync<T>(T);



////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl_unpack!(UnsafeSendSync | AsRef, AsMut, Deref, DerefMut, From);


unsafe impl<T> Send for UnsafeSendSync<T> {}
unsafe impl<T> Sync for UnsafeSendSync<T> {}


impl<T> UnsafeSendSync<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }

    pub fn unwrap(self) -> T {
        self.0
    }

    pub fn as_ptr(&self) -> *const T {
        &self.0
    }

    pub unsafe fn as_ref_mut_ptr(&self) -> *mut T {
        &self.0 as *const _ as *mut _
    }
}
