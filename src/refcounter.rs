use std::alloc::Layout;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::convert::{AsMut, AsRef};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{AddAssign, Deref, DerefMut, SubAssign};
/// [RefCounter](Self) is a data-structure designed specifically for
/// internal use in [`UniquePointer`](crate::UniquePointer) allowing reference counts to be
/// shared across clones of [`UniquePointer`](crate::UniquePointer).
///
/// [RefCounter](Self) uses relatively obscure rust techniques under the
/// hood to allow writing in non-mut references in strategic occasions
/// such as incrementing its reference count within its [`Clone`]
/// implementation.
///
/// Finally, [`write`](RefCounter::write), [`reset`](RefCounter::reset),
/// [`incr`](RefCounter::incr), [`incr_by`](RefCounter::incr_by),
/// [`decr`](RefCounter::decr), [`decr_by`](RefCounter::decr_by) allows `RefCounter`
/// instances to modify non-mut instances [`&RefCounter`](std#primitive.reference.html) of
/// [RefCounter](Self) such that implementors don't need to resort to
/// [`UniquePointer::unlock_reference`](crate::UniquePointer::unlock_reference).
pub struct RefCounter {
    data: *mut usize,
}

impl RefCounter {
    /// `new` creates a new [`RefCounter`](Self) with its internal state
    /// equivalent to zero.
    pub fn null() -> RefCounter {
        RefCounter {
            data: std::ptr::null_mut::<usize>(),
        }
    }

    /// `new` creates a new [`RefCounter`](Self) with the value 1
    pub fn new() -> RefCounter {
        let mut ref_counter = RefCounter::null();
        ref_counter.incr();
        ref_counter
    }

    /// `reset` resets a [`RefCounter`](Self) to one which is the equivalent
    /// state of a [`new`](RefCounter::new).
    pub fn reset(&self) {
        let mut up = unsafe { self.meta_mut() };
        up.write(1);
    }

    /// `incr` increments the `RefCounter` by one
    pub fn incr(&self) {
        let mut up = unsafe { self.meta_mut() };
        up.incr_by(1);
    }

    /// `incr_by` increments the `RefCounter`
    pub fn incr_by(&self, by: usize) {
        let mut up = unsafe { self.meta_mut() };
        up.write(up.read() + by);
    }

    /// `decr` decrements the `RefCounter` by one
    pub fn decr(&self) {
        let mut up = unsafe { self.meta_mut() };
        up.decr_by(1);
    }

    /// `decr_by` decrements the `RefCounter`
    pub fn decr_by(&self, by: usize) {
        let mut up = unsafe { self.meta_mut() };
        let data = up.read();
        if data >= by {
            up.write(data - by);
        }
    }

    /// `drain` deallocates the memory used by a [`RefCounter`](Self)
    /// resetting its internals so as to behave as though it has been
    /// written `0`.
    pub fn drain(&mut self) {
        if !self.data.is_null() {
            unsafe {
                self.data.drop_in_place();
                self.alloc();
            }
        }
    }

    pub fn read(&self) -> usize {
        if self.data.is_null() {
            0
        } else {
            let mut ptr = self.cast_const();
            unsafe { ptr.read() }
        }
    }

    fn alloc(&self) {
        if !self.data.is_null() {
            return;
        }

        let layout = Layout::new::<usize>();
        let ptr = unsafe {
            let ptr = std::alloc::alloc(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            ptr as *mut usize
        };
        let mut up = unsafe { self.meta_mut() };
        up.data = ptr;
        up.write(1);
    }

    /// `write` writes a [`usize`] into a [`RefCounter`](Self) as opposed to
    /// incrementing or decrementing it.
    pub fn write(&mut self, data: usize) {
        self.alloc();
        let mut ptr = self.cast_mut();
        unsafe {
            ptr.write(data);
        }
    }

    /// `inner_ref` returns a reference to the internal data of a
    /// [`RefCounter`]. Writing to the memory area if not already
    /// allocated.
    pub fn inner_ref<'c>(&self) -> &'c usize {
        if self.data.is_null() {
            &0
        } else {
            let ptr = self.cast_const();
            unsafe { &*ptr }
        }
    }

    /// `inner_mut` returns a mutable reference to the internal data
    /// of a [`RefCounter`]. Writing to the memory area if not already
    /// allocated.
    pub fn inner_mut<'c>(&mut self) -> &'c mut usize {
        if self.data.is_null() {
            self.write(0);
        }
        let mut ptr = self.cast_mut();
        unsafe { &mut *ptr }
    }
}
impl RefCounter {
    // private methods
    fn cast_mut(&self) -> *mut usize {
        self.data
    }

    fn cast_const(&self) -> *const usize {
        self.data.cast_const()
    }
}
impl From<usize> for RefCounter {
    fn from(refs: usize) -> RefCounter {
        let mut ref_counter = RefCounter::new();
        ref_counter.write(refs);
        ref_counter
    }
}
impl AsRef<usize> for RefCounter {
    fn as_ref(&self) -> &usize {
        self.inner_ref()
    }
}
impl AsMut<usize> for RefCounter {
    fn as_mut(&mut self) -> &mut usize {
        if self.data.is_null() {
            self.write(0);
        }
        let mut ptr = self.cast_mut();
        unsafe { &mut *ptr }
    }
}
impl Deref for RefCounter {
    type Target = usize;

    fn deref(&self) -> &usize {
        self.inner_ref()
    }
}
impl DerefMut for RefCounter {
    fn deref_mut(&mut self) -> &mut usize {
        self.inner_mut()
    }
}

impl Drop for RefCounter {
    fn drop(&mut self) {
        self.drain()
    }
}

impl Clone for RefCounter {
    fn clone(&self) -> RefCounter {
        let mut clone = RefCounter::new();
        clone.data = self.data;
        clone
    }
}

impl std::fmt::Debug for RefCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            [
                format!("RefCounter@"),
                format!("{:016x}", self.data.addr()),
                format!("[data={}]", self.read()),
            ]
            .join("")
        )
    }
}
impl std::fmt::Display for RefCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.read())
    }
}

#[allow(invalid_reference_casting)]
impl<'c> RefCounter {
    /// `meta_mut` is an unsafe method that turns a "self reference"
    /// into a mutable "self reference"
    unsafe fn meta_mut(&'c self) -> &'c mut RefCounter {
        unsafe {
            let ptr = self.meta_mut_ptr();
            let mut up = &mut *ptr;
            std::mem::transmute::<&mut RefCounter, &'c mut RefCounter>(up)
        }
    }

    /// `meta_mut_ptr` is an unsafe method that turns a [`*mut UniquePointer`](crate::UniquePointer) from a "self reference"
    unsafe fn meta_mut_ptr(&self) -> *mut RefCounter {
        let ptr = self as *const RefCounter;
        unsafe {
            let ptr: *mut RefCounter =
                std::mem::transmute::<*const RefCounter, *mut RefCounter>(ptr);
            ptr
        }
    }
}

impl AddAssign<usize> for RefCounter {
    fn add_assign(&mut self, other: usize) {
        self.incr_by(other)
    }
}

impl SubAssign<usize> for RefCounter {
    fn sub_assign(&mut self, other: usize) {
        self.decr_by(other)
    }
}

impl PartialOrd<usize> for RefCounter {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.read().partial_cmp(other)
    }
}

impl PartialEq<usize> for RefCounter {
    fn eq(&self, other: &usize) -> bool {
        self.read().eq(other)
    }
}

impl PartialOrd for RefCounter {
    fn partial_cmp(&self, other: &RefCounter) -> Option<Ordering> {
        self.read().partial_cmp(other.inner_ref())
    }
}

impl Ord for RefCounter {
    fn cmp(&self, other: &RefCounter) -> Ordering {
        self.read().cmp(other.inner_ref())
    }
}

impl PartialEq for RefCounter {
    fn eq(&self, other: &RefCounter) -> bool {
        self.read().eq(other.inner_ref())
    }
}

impl Eq for RefCounter {}

impl Hash for RefCounter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.read().hash(state)
    }
}
