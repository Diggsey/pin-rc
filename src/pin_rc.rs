use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref, RefMut, BorrowError, BorrowMutError};
use std::mem::Pin;
use std::marker::Unpin;
use std::ops::Deref;
use std::fmt;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PinRc<T: ?Sized> {
    inner: Rc<RefCell<T>>
}

pub struct PinWeak<T: ?Sized> {
    inner: Weak<RefCell<T>>
}

pub struct PinRef<'a, T: ?Sized + 'a> {
    inner: Ref<'a, T>
}

pub struct PinRefMut<'a, T: ?Sized + 'a> {
    inner: RefMut<'a, T>
}

impl<T> PinRc<T> {
    /// Allocate memory on the heap, move the data into it and pin it.
    pub fn new(data: T) -> PinRc<T> {
        PinRc { inner: Rc::new(RefCell::new(data)) }
    }
}

impl<T: Unpin + ?Sized> PinRc<T> {
    pub fn safe_unpin(this: PinRc<T>) -> Rc<RefCell<T>> {
        this.inner
    }
}

impl<T: ?Sized> PinRc<T> {
    pub fn into_raw(this: Self) -> *const RefCell<T> {
        Rc::into_raw(this.inner)
    }

    pub unsafe fn from_raw(ptr: *const RefCell<T>) -> Self {
        PinRc { inner: Rc::from_raw(ptr) }
    }

    /// Convert this PinRc into an unpinned Rc.
    ///
    /// This function is unsafe. Users must guarantee that data is never
    /// moved out of the Rc.
    #[inline]
    pub unsafe fn unpin(this: PinRc<T>) -> Rc<RefCell<T>> {
        this.inner
    }

    #[inline]
    pub fn downgrade(this: &Self) -> PinWeak<T> {
        PinWeak { inner: Rc::downgrade(&this.inner) }
    }

    #[inline]
    pub fn weak_count(this: &Self) -> usize {
        Rc::weak_count(&this.inner)
    }

    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        Rc::strong_count(&this.inner)
    }

    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Rc::ptr_eq(&this.inner, &other.inner)
    }

    #[inline]
    pub fn borrow(&self) -> PinRef<T> {
        PinRef { inner: self.inner.borrow() }
    }

    #[inline]
    pub fn borrow_mut(&self) -> PinRefMut<T> {
        PinRefMut { inner: self.inner.borrow_mut() }
    }

    #[inline]
    pub fn try_borrow(&self) -> Result<PinRef<T>, BorrowError> {
        Ok(PinRef { inner: self.inner.try_borrow()? })
    }

    #[inline]
    pub fn try_borrow_mut(&self) -> Result<PinRefMut<T>, BorrowMutError> {
        Ok(PinRefMut { inner: self.inner.try_borrow_mut()? })
    }
}

impl<T: ?Sized> Clone for PinRc<T> {
    #[inline]
    fn clone(&self) -> Self {
        PinRc { inner: self.inner.clone() }
    }
}

impl<T> From<T> for PinRc<T> {
    #[inline]
    fn from(t: T) -> Self {
        PinRc::new(t)
    }
}

impl<T> From<Rc<RefCell<T>>> for PinRc<T> {
    #[inline]
    fn from(inner: Rc<RefCell<T>>) -> Self {
        PinRc { inner }
    }
}

impl<'a, T: ?Sized> PinRef<'a, T> {
    #[inline]
    pub fn clone(this: &Self) -> Self {
        PinRef { inner: Ref::clone(&this.inner) }
    }
}

impl<'a, T> Deref for PinRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T: ?Sized> PinRefMut<'a, T> {
    #[inline]
    pub fn as_pin(&mut self) -> Pin<T> {
        unsafe { Pin::new_unchecked(&mut *self.inner) }
    }
    #[inline]
    pub unsafe fn get_mut(this: &mut Self) -> &mut T {
        &mut *this.inner
    }
    pub fn map<U: ?Sized, F>(orig: Self, f: F) -> PinRefMut<'a, U>
        where F: FnOnce(Pin<T>) -> Pin<U>
    {
        PinRefMut { inner: RefMut::map(orig.inner, |v| {
            let pin_v = unsafe { Pin::new_unchecked(v) };
            let mut pin_u = f(pin_v);
            let u = unsafe { Pin::get_mut(&mut pin_u) };
            unsafe { &mut *(u as *mut U) }
        }) }
    }
}

impl<'a, T> Deref for PinRefMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<T: ?Sized> PinWeak<T> {
    #[inline]
    pub fn upgrade(&self) -> Option<PinRc<T>> {
        self.inner.upgrade().map(|inner| PinRc { inner })
    }
}

impl<T: ?Sized> Clone for PinWeak<T> {
    /// Makes a clone of the `PinWeak` that points to the same value.
    #[inline]
    fn clone(&self) -> PinWeak<T> {
        PinWeak { inner: self.inner.clone() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for PinWeak<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T> Default for PinWeak<T> {
    /// Constructs a new `PinWeak<T>`, allocating memory for `T` without initializing
    /// it. Calling [`upgrade`] on the return value always gives [`None`].
    fn default() -> PinWeak<T> {
        PinWeak { inner: Weak::default() }
    }
}
