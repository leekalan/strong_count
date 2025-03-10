use std::{
    marker::{PhantomData, Unsize},
    ops::{CoerceUnsized, Deref, DispatchFromDyn},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

#[repr(C)]
struct AscInner<T: ?Sized> {
    count: AtomicUsize,
    data: T,
}

pub struct Asc<T: ?Sized> {
    ptr: NonNull<AscInner<T>>,
    _marker: PhantomData<AscInner<T>>,
}

impl<T: ?Sized> Drop for Asc<T> {
    fn drop(&mut self) {
        if self.inner().count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        let _drop = unsafe { Box::from_raw(self.ptr.as_mut()) };
    }
}

impl<T> Asc<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self::new_with_count(data)
    }

    #[inline]
    pub fn new_with_count(data: T) -> Asc<T> {
        let ptr = Box::leak(Box::new(AscInner {
            count: AtomicUsize::new(1),
            data,
        }));

        Asc {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Asc<T> {
    #[inline(always)]
    fn inner(&self) -> &AscInner<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        unsafe { &mut (*self.ptr.as_ptr()).data }
    }
}

impl<T: ?Sized> Clone for Asc<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.inner().count.fetch_add(1, Ordering::Relaxed);

        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Deref for Asc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

unsafe impl<T: ?Sized> Sync for Asc<T> {}
unsafe impl<T: ?Sized> Send for Asc<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Asc<U>> for Asc<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Asc<U>> for Asc<T> {}
