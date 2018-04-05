use std::sync::{
    Arc, Weak, RwLock, RwLockReadGuard, RwLockWriteGuard, LockResult, PoisonError, TryLockError,
    TryLockResult
};
use std::mem::Pin;
use std::marker::Unpin;
use std::ops::Deref;
use std::fmt;

#[derive(Default, Debug)]
pub struct PinArc<T: ?Sized> {
    inner: Arc<RwLock<T>>
}

pub struct PinWeak<T: ?Sized> {
    inner: Weak<RwLock<T>>
}

pub struct PinRwLockReadGuard<'a, T: ?Sized + 'a> {
    inner: RwLockReadGuard<'a, T>
}

pub struct PinRwLockWriteGuard<'a, T: ?Sized + 'a> {
    inner: RwLockWriteGuard<'a, T>
}

impl<T> PinArc<T> {
    /// Allocate memory on the heap, move the data into it and pin it.
    pub fn new(data: T) -> PinArc<T> {
        PinArc { inner: Arc::new(RwLock::new(data)) }
    }
}

impl<T: Unpin + ?Sized> PinArc<T> {
    pub fn safe_unpin(this: PinArc<T>) -> Arc<RwLock<T>> {
        this.inner
    }
}

impl<T: ?Sized> PinArc<T> {
    pub fn into_raw(this: Self) -> *const RwLock<T> {
        Arc::into_raw(this.inner)
    }

    pub unsafe fn from_raw(ptr: *const RwLock<T>) -> Self {
        PinArc { inner: Arc::from_raw(ptr) }
    }

    /// Convert this PinArc into an unpinned Arc.
    ///
    /// This function is unsafe. Users must guarantee that data is never
    /// moved out of the Arc.
    #[inline]
    pub unsafe fn unpin(this: PinArc<T>) -> Arc<RwLock<T>> {
        this.inner
    }

    #[inline]
    pub fn downgrade(this: &Self) -> PinWeak<T> {
        PinWeak { inner: Arc::downgrade(&this.inner) }
    }

    #[inline]
    pub fn weak_count(this: &Self) -> usize {
        Arc::weak_count(&this.inner)
    }

    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        Arc::strong_count(&this.inner)
    }

    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Arc::ptr_eq(&this.inner, &other.inner)
    }

    #[inline]
    pub fn read(&self) -> LockResult<PinRwLockReadGuard<T>> {
        match self.inner.read() {
            Ok(inner) => Ok(PinRwLockReadGuard { inner }),
            Err(p) => Err(PoisonError::new(PinRwLockReadGuard { inner: p.into_inner() })),
        }
    }

    #[inline]
    pub fn write(&self) -> LockResult<PinRwLockWriteGuard<T>> {
        match self.inner.write() {
            Ok(inner) => Ok(PinRwLockWriteGuard { inner }),
            Err(p) => Err(PoisonError::new(PinRwLockWriteGuard { inner: p.into_inner() })),
        }
    }

    #[inline]
    pub fn try_read(&self) -> TryLockResult<PinRwLockReadGuard<T>> {
        match self.inner.try_read() {
            Ok(inner) => Ok(PinRwLockReadGuard { inner }),
            Err(TryLockError::Poisoned(p)) => Err(TryLockError::Poisoned(PoisonError::new(
                PinRwLockReadGuard { inner: p.into_inner() }
            ))),
            Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
        }
    }

    #[inline]
    pub fn try_write(&self) -> TryLockResult<PinRwLockWriteGuard<T>> {
        match self.inner.try_write() {
            Ok(inner) => Ok(PinRwLockWriteGuard { inner }),
            Err(TryLockError::Poisoned(p)) => Err(TryLockError::Poisoned(PoisonError::new(
                PinRwLockWriteGuard { inner: p.into_inner() }
            ))),
            Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
        }
    }

    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }
}

impl<T: ?Sized> Clone for PinArc<T> {
    #[inline]
    fn clone(&self) -> Self {
        PinArc { inner: self.inner.clone() }
    }
}

impl<T> From<T> for PinArc<T> {
    #[inline]
    fn from(t: T) -> Self {
        PinArc::new(t)
    }
}

impl<T> From<Arc<RwLock<T>>> for PinArc<T> {
    #[inline]
    fn from(inner: Arc<RwLock<T>>) -> Self {
        PinArc { inner }
    }
}

impl<'a, T> Deref for PinRwLockReadGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T: ?Sized> PinRwLockWriteGuard<'a, T> {
    #[inline]
    pub fn as_pin(&mut self) -> Pin<T> {
        unsafe { Pin::new_unchecked(&mut *self.inner) }
    }
    #[inline]
    pub unsafe fn get_mut(this: &mut Self) -> &mut T {
        &mut *this.inner
    }
}

impl<'a, T> Deref for PinRwLockWriteGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<T: ?Sized> PinWeak<T> {
    #[inline]
    pub fn upgrade(&self) -> Option<PinArc<T>> {
        self.inner.upgrade().map(|inner| PinArc { inner })
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
