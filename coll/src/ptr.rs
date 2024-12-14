use std::ptr::NonNull;

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[repr(transparent)]
pub struct Ptr<T> {
    value: NonNull<T>,
}

#[repr(transparent)]
pub struct OwnedPtr<T> {
    value: NonNull<T>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: NonNull::new(Box::into_raw(Box::new(value))).unwrap(),
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &* self.value.as_ptr() }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.value.as_ptr() }
    }

    pub fn ptr(&self) -> Ptr<T> {
        Ptr { value: self.value }
    }
}

impl<T> Drop for OwnedPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.value.as_ptr());
        }
    }
}

impl<T> Ptr<T> {
    pub fn as_ref(&self) -> &T {
        unsafe { &* self.value.as_ptr() }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.value.as_ptr() }
    }
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self { value: self.value }
    }
}

impl<T> Copy for Ptr<T> {}
