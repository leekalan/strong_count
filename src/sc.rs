use std::{
    cell::Cell,
    marker::{PhantomData, Unsize},
    ops::{CoerceUnsized, Deref, DispatchFromDyn},
    ptr::NonNull,
};

#[repr(C)]
struct ScInner<T: ?Sized> {
    count: Cell<usize>,
    data: T,
}

pub struct Sc<T: ?Sized> {
    ptr: NonNull<ScInner<T>>,
    _marker: PhantomData<ScInner<T>>,
}

impl<T: ?Sized> Drop for Sc<T> {
    #[inline]
    fn drop(&mut self) {
        if self.inner().dec_count() != 1 {
            return;
        }

        let _drop = unsafe { Box::from_raw(self.ptr.as_mut()) };
    }
}

impl<T> Sc<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self::new_with_count(data)
    }

    #[inline]
    pub fn new_with_count(data: T) -> Sc<T> {
        let ptr = Box::leak(Box::new(ScInner {
            count: Cell::new(1),
            data,
        }));

        Sc {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Sc<T> {
    #[inline(always)]
    fn inner(&self) -> &ScInner<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        unsafe { &mut (*self.ptr.as_ptr()).data }
    }
}

impl<T: ?Sized> Clone for Sc<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.inner().inc_count();

        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> ScInner<T> {
    #[inline(always)]
    fn inc_count(&self) {
        self.count.set(self.count.get() + 1);
    }

    #[inline(always)]
    fn dec_count(&self) -> usize {
        let v = self.count.get();
        self.count.set(v - 1);
        v
    }
}

impl<T: ?Sized> Deref for Sc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

impl<T: ?Sized> AsRef<T> for Sc<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> !Sync for Sc<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Sc<U>> for Sc<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Sc<U>> for Sc<T> {}
