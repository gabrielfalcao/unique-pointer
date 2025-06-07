use std::alloc::Layout;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::convert::{AsMut, AsRef};
use std::fmt::{Debug, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use crate::{RefCounter, UniquePointee};

/// experimental data structure that makes extensive use of unsafe
/// rust to provide a shared pointer throughout the runtime of a rust
/// program as transparently as possible.
///
/// [`UniquePointer`]'s design's purpose is two-fold:
///
/// - Leverage the implementation of circular data structures such as
/// Lisp cons cells.
///
/// - Making easier the task of practicing the implementation of basic
/// computer science data-structures (e.g.: Binary Trees, Linked Lists
/// etc) such that the concept of pointer is as close to C as possible
/// in terms of developer experience and so when a CS teacher speaks
/// in terms of pointers, students can use [`UniquePointer`] in their
/// data-structures knowing that cloning their data-structures also
/// means cloning the pointers transparently.
///
/// In fact, the author designed `UniquePointer` while studying the
/// MIT CourseWare material of professor Erik Demaine in addition to
/// studying lisp "cons" cells.
///
/// To this point the author reiterates: `UniquePointer` is an
/// **experimental** data-structure designed primarily as a
/// building-block of other data-structures in rust.
///
/// `UniquePointer` provides the methods [`UniquePointer::cast_mut`]
/// and [`UniquePointer::cast_const`] not unlike those of raw
/// pointers, and also implements the methods
/// [`UniquePointer::as_ref`] and [`UniquePointer::as_mut`] with a
/// signature compatible with that of the [`AsRef`] and [`AsMut`]
/// traits such that users of raw pointers can migrate to
/// [`UniquePointer`] without much difficulty.
///
/// `UniquePointer` is designed a way such that Enums and Structs
/// using `UniquePointer` can safely clone `UniquePointer` while the
/// memory address and provenance of its value is shared.
///
/// [`UniquePointer`] is able to extend lifetimes because it maintains
/// its own reference counting outside of the rust compiler.
///
/// Reference Counting is provided by [`RefCounter`] which uses unsafe
/// rust to ensure that ref counts are shared across cloned objects
/// memory.
///
/// Both [`UniquePointer`] and [`RefCounter`] use relatively obscure
/// rust techniques under the hood to allow writing in non-mut
/// references in strategic occasions such as incrementing its
/// reference count within its [`Clone`] implementation.
///
/// UniquePointer only supports [`Sized`] types, that is, [Zero-Sized
/// Types](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)
/// (ZSTs) are not supported.
///
/// Example
///
/// ```
/// use unique_pointer::UniquePointer;
///
/// fn create_unique_pointer<'a>() -> UniquePointer<&'a str> {
///     UniquePointer::from("string")
/// }
/// let mut value: UniquePointer<&'_ str> = create_unique_pointer();
///
/// assert_eq!(value.is_null(), false);
///
/// assert_eq!(value.is_allocated(), true);
/// assert!(value.addr() > 0, "address should not be null");
/// assert_eq!(value.is_written(), true);
/// assert_eq!(value.inner_ref(), &"string");
///
/// assert_eq!(value.read(), "string");
/// assert_eq!(value.as_ref(), Some(&"string"));
/// ```
///
/// # Caveats of `UniquePointer`:
///
/// - Only supports types that implement [`Debug`]
/// - Does not support [ZSTs](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts) (Zero-Sized Types)
/// - [`UniquePointer`] **IS NOT THREAD SAFE**
///
pub struct UniquePointer<T: UniquePointee> {
    mut_addr: usize,
    mut_ptr: *mut T,
    refs: RefCounter,
    alloc: bool,
    is_copy: bool,
    written: bool,
}

impl<'c, T: UniquePointee + 'c> UniquePointer<T> {
    /// creates a NULL `UniquePointer` ready to be written via [`write`].
    pub fn null() -> UniquePointer<T> {
        UniquePointer {
            mut_addr: 0,
            mut_ptr: std::ptr::null_mut::<T>(),
            refs: RefCounter::new(),
            written: false,
            alloc: false,
            is_copy: false,
        }
    }

    /// creates a new `UniquePointer` by effectively
    /// reading the value referenced by [`src`]
    ///
    pub fn from_ref(src: &T) -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.write_ref(src);
        up
    }

    /// `from_ref_mut` creates a new `UniquePointer` by effectively
    /// reading the value referenced by `src`
    ///
    pub fn from_ref_mut(src: &mut T) -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.write_ref_mut(src);
        up
    }

    /// is designed for use within the [`Clone`] implementation
    /// of `UniquePointer`.
    ///
    /// The [`copy`] method creates a NULL `UniquePointer` flagged as
    /// [`is_copy`] such that a double-free does not happen in
    /// [`dealloc`].
    fn copy() -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.is_copy = true;
        up
    }

    /// produces a copy of a `UniquePointer` which is not a copy in
    /// the sense that [`UniquePointer::is_copy`] returns true.
    ///
    /// Because of that rationale a double-free occurs if there are
    /// two or more "containers" (e.g.: [`struct`]s and [`enum`]s)
    /// implementing [`Drop`] and holding the same propagated
    /// `UniquePointer` instance. For this reason
    /// [`UniquePointer::propagate`] is unsafe.
    ///
    /// [`UniquePointer::propagate`] can be relatively observed as a
    /// drop-in replacement to [`UniquePointer::clone`] for cases
    /// when, for instance, swapping `UniquePointer` "instances"
    /// between instances of `UniquePointer`-containing (structs,
    /// enums and/or unions) is desired.
    ///
    /// Example
    ///
    /// ```
    /// use unique_pointer::UniquePointer;
    /// use std::fmt::Debug;
    /// use std::cmp::PartialEq;
    ///
    /// #[derive(Clone, Debug)]
    /// pub struct BinaryTreeNode<T: Debug> {
    ///     pub item: T,
    ///     pub parent: UniquePointer<BinaryTreeNode<T>>,
    ///     pub left: UniquePointer<BinaryTreeNode<T>>,
    ///     pub right: UniquePointer<BinaryTreeNode<T>>,
    /// }
    /// impl<T: Debug> BinaryTreeNode<T> {
    ///     pub fn new(item: T) -> BinaryTreeNode<T> {
    ///         BinaryTreeNode {
    ///             item,
    ///             parent: UniquePointer::null(),
    ///             left: UniquePointer::null(),
    ///             right: UniquePointer::null(),
    ///         }
    ///     }
    ///
    ///     pub fn rotate_left(&mut self) {
    ///         if self.parent.is_null() {
    ///             if self.right.is_not_null() {
    ///                 self.parent = unsafe { self.right.propagate() };
    ///                 self.right = UniquePointer::null();
    ///             }
    ///         }
    ///     }
    ///
    ///     pub fn set_parent(&mut self, parent: &mut BinaryTreeNode<T>) {
    ///         self.parent = UniquePointer::read_only(parent);
    ///     }
    ///
    ///     pub fn set_left(&mut self, left: &mut BinaryTreeNode<T>) {
    ///         left.set_parent(self);
    ///         self.left = UniquePointer::read_only(left);
    ///     }
    ///
    ///     pub fn set_right(&mut self, right: &mut BinaryTreeNode<T>) {
    ///         right.set_parent(self);
    ///         self.right = UniquePointer::read_only(right);
    ///     }
    /// }
    ///
    /// let mut node_a = BinaryTreeNode::new("A");
    /// let mut node_b = BinaryTreeNode::new("B");
    /// let mut node_c = BinaryTreeNode::new("C");
    /// node_a.set_left(&mut node_b);
    /// node_a.set_right(&mut node_c);
    ///
    /// ```
    pub unsafe fn propagate(&self) -> UniquePointer<T> {
        self.incr_ref();
        let mut back_node = UniquePointer::<T>::null();
        back_node.set_mut_ptr(self.mut_ptr, false);
        back_node.refs = self.refs.clone();
        back_node.alloc = self.alloc;
        back_node.written = self.written;
        back_node
    }

    /// calls [`copy_from_ref`] to create a *read-only* `UniquePointer` from a
    /// reference of `T`, useful for iterating over self-referential
    /// data structures.
    ///
    /// Example:
    ///
    /// ```

    /// ```
    pub fn read_only(data: &T) -> UniquePointer<T> {
        UniquePointer::copy_from_ref(data, 1)
    }

    /// calls [`copy_from_mut_ptr`] to create a *read-only*
    /// `UniquePointer` from a reference of `T`, useful for
    /// iterating over self-referential data structures that use
    /// [`RefCounter`] to count refs.
    ///
    /// Note: [`read_only`] might be a better alternative when `T` is
    /// a data structure that does not use [`RefCounter`].
    pub fn copy_from_ref(data: &T, refs: usize) -> UniquePointer<T> {
        let ptr = (data as *const T).cast_mut();
        UniquePointer::copy_from_mut_ptr(ptr, refs)
    }

    /// creates a *read-only* `UniquePointer`
    /// from a reference of `T`, useful for iterating over
    /// self-referential data structures that use [`RefCounter`] to
    /// count refs.
    ///
    /// Note: [`read_only`] might be a better alternative when `T` is
    /// a data structure that does not use [`RefCounter`].
    pub fn copy_from_mut_ptr(ptr: *mut T, refs: usize) -> UniquePointer<T> {
        let addr = UniquePointer::provenance_of_mut_ptr(ptr);
        let refs = RefCounter::from(refs);
        UniquePointer {
            mut_addr: addr,
            mut_ptr: ptr,
            refs: refs,
            written: true,
            alloc: true,
            is_copy: true,
        }
    }

    /// returns the value containing both the provenance and
    /// memory address of a pointer
    pub fn addr(&self) -> usize {
        self.mut_addr
    }

    /// returns the reference count of a `UniquePointer`
    pub fn refs(&self) -> usize {
        *self.refs
    }

    /// returns true if the `UniquePointer` is NULL.
    pub fn is_null(&self) -> bool {
        let mut_is_null = self.mut_ptr.is_null();
        if mut_is_null {
            assert!(self.mut_addr == 0);
        } else {
            assert!(self.mut_addr != 0);
        }
        let is_null = mut_is_null;
        is_null
    }

    /// returns true if the `UniquePointer` is not
    /// NULL. [`is_not_null`] is a idiomatic shortcut to negating a call
    /// to [`is_null`] such that the negation is less likely to be
    /// clearly visible.
    pub fn is_not_null(&self) -> bool {
        !self.is_null()
    }

    /// returns true if the `UniquePointer` is not a
    /// copy. [`is_not_copy`] is a idiomatic shortcut to negating a call
    /// to [`is_copy`] such that the negation is less likely to be
    /// clearly visible.
    pub fn is_not_copy(&self) -> bool {
        !self.is_copy
    }

    /// returns true if the `UniquePointer` is not NULL
    /// and is not flagged as a copy, meaning it can be deallocated
    /// without concern for double-free.
    pub fn can_dealloc(&self) -> bool {
        self.alloc && self.is_not_copy() && self.is_not_null()
    }

    /// returns true if the `UniquePointer` has been
    /// allocated and therefore is no longer a NULL pointer.
    pub fn is_allocated(&self) -> bool {
        let is_allocated = self.is_not_null() && self.alloc;
        is_allocated
    }

    /// returns true if the `UniquePointer` has been written to
    pub fn is_written(&self) -> bool {
        let is_written = self.is_allocated() && self.written;
        is_written
    }

    /// returns true if a `UniquePointer` is a "copy" of
    /// another `UniquePointer` in the sense that dropping or
    /// "hard-deallocating" said `UniquePointer` does not incur a
    /// double-free.
    pub fn is_copy(&self) -> bool {
        self.is_copy
    }

    /// allocates memory in a null `UniquePointer`
    pub fn alloc(&mut self) {
        if self.is_allocated() {
            return;
        }

        let layout = Layout::new::<T>();
        let mut_ptr = unsafe {
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            ptr as *mut T
        };
        self.set_mut_ptr(mut_ptr, false);
        self.alloc = true;
    }

    /// compatibility API to a raw mut pointer's [`pointer::cast_mut`].
    pub fn cast_mut(&self) -> *mut T {
        if self.is_null() {
            panic!("{:#?}", self);
        } else {
            self.mut_ptr
        }
    }

    /// compatibility API to a raw const pointer's [`pointer::cast_const`].
    pub fn cast_const(&self) -> *const T {
        if self.is_null() {
            panic!("{:#?}", self);
        } else {
            self.mut_ptr.cast_const()
        }
    }

    /// allocates memory and writes the given value into the
    /// newly allocated area.
    pub fn write(&mut self, data: T) {
        self.alloc();

        unsafe {
            self.mut_ptr.write(data);
        }

        self.written = true;
    }

    /// takes a mutable reference to a value and
    /// writes to a `UniquePointer`
    pub fn write_ref_mut(&mut self, data: &mut T) {
        self.alloc();
        unsafe {
            let ptr = data as *mut T;
            ptr.copy_to(self.mut_ptr, 1);
        };
        self.written = true;
    }

    /// takes a read-only reference to a value and
    /// writes to a `UniquePointer`
    pub fn write_ref(&mut self, data: &T) {
        self.alloc();
        unsafe {
            let ptr = data as *const T;
            ptr.copy_to(self.mut_ptr, 1);
        };
        self.written = true;
    }

    /// swaps the memory addresses storing `T` with other `UniquePointer`
    pub fn swap(&mut self, other: &mut Self) {
        if self.is_null() && other.is_null() {
            return;
        }
        if self.mut_ptr.is_null() {
            self.alloc();
        }
        if other.mut_ptr.is_null() {
            other.alloc();
        }
        unsafe {
            self.mut_ptr.swap(other.mut_ptr);
        }
    }

    /// reads data from memory `UniquePointer`. Panics if
    /// the pointer is either null or allocated but never written to.
    pub fn read(&self) -> T {
        if !self.is_written() {
            panic!("{:#?} not written", self);
        }
        let ptr = self.cast_const();
        unsafe { ptr.read() }
    }

    /// reads data from memory `UniquePointer`
    pub fn try_read(&self) -> Option<T> {
        if self.is_written() {
            Some(self.read())
        } else {
            None
        }
    }

    /// obtains a read-only reference to the value inside
    /// `UniquePointer` but does not increment references
    pub fn inner_ref(&self) -> &'c T {
        if self.mut_ptr.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        unsafe { std::mem::transmute::<&T, &'c T>(&*self.cast_const()) }
    }

    /// obtains a mutable reference to the value inside
    /// `UniquePointer` but does not increment references
    pub fn inner_mut(&mut self) -> &'c mut T {
        if self.mut_ptr.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        unsafe { std::mem::transmute::<&mut T, &'c mut T>(&mut *self.mut_ptr) }
    }

    /// compatibility layer to [`std::pointer::as_ref`]
    pub fn as_ref(&self) -> Option<&'c T> {
        if self.is_written() {
            Some(self.inner_ref())
        } else {
            None
        }
    }

    /// compatibility layer to [`std::pointer::as_mut`]
    pub fn as_mut(&mut self) -> Option<&'c mut T> {
        if self.is_written() {
            Some(self.inner_mut())
        } else {
            None
        }
    }

    /// deallocates a `UniquePointer`.
    ///
    /// The [`soft`] boolean argument indicates whether the
    /// `UniquePointer` should have its reference count decremented or
    /// deallocated immediately.
    ///
    /// During "soft" deallocation (`soft=true`) calls to `dealloc`
    /// only really deallocate memory when the reference gets down to
    /// zero, until then each `dealloc(true)` call simply decrements
    /// the reference count.
    ///
    /// Conversely during "hard" deallocation (`soft=false`) the
    /// UniquePointer in question gets immediately deallocated,
    /// possibly incurring a double-free or causing Undefined
    /// Behavior.
    pub fn dealloc(&mut self, soft: bool) {
        if self.is_null() {
            return;
        }
        if soft && self.refs > 0 {
            self.decr_ref();
        } else {
            self.free();
        }
    }

    /// sets the internal raw pointer of a `UniquePointer`.
    ///
    /// Prior to setting the new pointer, it checks whether the
    /// internal pointer is non-null and matches its provenance
    /// address, such that "copies" do not incur a double-free.
    ///
    /// When [`ptr`] is a NULL pointer and the internal pointer of
    /// `UniquePointer` in question is NOT NULL, then it is
    /// deallocated prior to setting it to NULL.
    fn set_mut_ptr(&mut self, ptr: *mut T, dealloc: bool) {
        if ptr.is_null() {
            if dealloc && self.is_allocated() {
                self.alloc = false;
                self.written = false;
                self.mut_addr = 0;
                let layout = Layout::new::<T>();
                unsafe {
                    std::alloc::dealloc(self.mut_ptr as *mut u8, layout);
                };
                self.mut_ptr = std::ptr::null_mut::<T>();
            }

            self.set_mut_addr(0);
        } else {
            self.set_mut_addr(UniquePointer::<T>::provenance_of_mut_ptr(ptr));
        }
        self.mut_ptr = ptr;
    }

    /// deallocates the memory used by `UniquePointer`
    /// once its references get down to zero.
    pub fn drop_in_place(&mut self) {
        self.dealloc(true);
    }

    fn set_mut_addr(&mut self, addr: usize) {
        self.mut_addr = addr;
    }

    /// is internally used by [`dealloc`] when the number of
    /// references gets down to zero in a "soft" deallocation and
    /// immediately in a "hard" deallocation.
    ///
    /// See [`dealloc`] for more information regarding the difference
    /// between "soft" and "hard" deallocation.
    fn free(&mut self) {
        if !self.can_dealloc() {
            return;
        }
        if !self.is_null() {
            self.set_mut_ptr(std::ptr::null_mut::<T>(), false);
            self.refs.drain();
        }
        self.alloc = false;
        self.written = false;
    }

    /// utility method to extend the lifetime
    /// of references of data created within a function.
    ///
    /// Example
    ///
    /// ```
    /// use unique_pointer::UniquePointer;
    ///
    /// pub struct Data<'r> {
    ///     value: &'r String,
    /// }
    /// impl <'r> Data<'r> {
    ///     pub fn new<T: std::fmt::Display>(value: T) -> Data<'r> {
    ///         let value = value.to_string();
    ///         Data {
    ///             value: UniquePointer::read_only(&value).extend_lifetime()
    ///         }
    ///     }
    /// }
    /// ```
    pub fn extend_lifetime<'t>(&self) -> &'t T {
        unsafe { std::mem::transmute::<&T, &'t T>(self.inner_ref()) }
    }

    /// utility method to extend the lifetime
    /// of references of data created within a function.
    ///
    /// Example
    ///
    /// ```
    /// use unique_pointer::UniquePointer;
    ///
    /// pub struct Data<'r> {
    ///     value: &'r mut String,
    /// }
    /// impl <'r> Data<'r> {
    ///     pub fn new<T: std::fmt::Display>(value: T) -> Data<'r> {
    ///         let value = value.to_string();
    ///         Data {
    ///             value: UniquePointer::read_only(&value).extend_lifetime_mut()
    ///         }
    ///     }
    /// }
    /// ```
    pub fn extend_lifetime_mut<'t>(&mut self) -> &'t mut T {
        unsafe { std::mem::transmute::<&mut T, &'t mut T>(self.inner_mut()) }
    }
}

impl<T: UniquePointee> UniquePointer<T> {
    /// helper method that returns the
    /// address and provenance of a const pointer
    pub fn provenance_of_const_ptr(ptr: *const T) -> usize {
        ptr.expose_provenance()
    }

    /// helper method that returns the
    /// address and provenance of a mut pointer
    pub fn provenance_of_mut_ptr(ptr: *mut T) -> usize {
        ptr.expose_provenance()
    }

    /// helper method that returns the
    /// address and provenance of a reference
    pub fn provenance_of_ref(ptr: &T) -> usize {
        (&raw const ptr).expose_provenance()
    }

    /// helper method that returns the
    /// address and provenance of a mutable reference
    pub fn provenance_of_mut(mut ptr: &mut T) -> usize {
        (&raw mut ptr).expose_provenance()
    }
}

#[allow(unused)]
impl<'c, T: UniquePointee + 'c> UniquePointer<T> {
    /// unsafe method that turns a "self reference"
    /// into a mutable "self reference"
    unsafe fn meta_mut(&'c self) -> &'c mut UniquePointer<T> {
        unsafe {
            let ptr = self.meta_mut_ptr();
            let up = &mut *ptr;
            std::mem::transmute::<&mut UniquePointer<T>, &'c mut UniquePointer<T>>(up)
        }
    }

    /// unsafe method that turns a [`*mut UniquePointer`] from a "self reference"
    unsafe fn meta_mut_ptr(&self) -> *mut UniquePointer<T> {
        let ptr = self as *const UniquePointer<T>;
        unsafe {
            let ptr: *mut UniquePointer<T> =
                std::mem::transmute::<*const UniquePointer<T>, *mut UniquePointer<T>>(ptr);
            ptr
        }
    }
}
#[allow(invalid_reference_casting)]
impl<T: UniquePointee> UniquePointer<T> {
    /// `incr_ref` uses unsafe rust to increment references of a
    /// non-mut reference to `UniquePointer`
    fn incr_ref(&self) {
        if self.is_null() {
            return;
        }
        unsafe {
            let ptr = self.meta_mut_ptr();
            let up = &mut *ptr;
            up.refs.incr();
        }
    }

    /// uses unsafe rust to decrement references of a
    /// non-mut reference to `UniquePointer`
    fn decr_ref(&self) {
        if self.refs == 0 {
            return;
        }
        unsafe {
            let ptr = self.meta_mut_ptr();
            let up = &mut *ptr;
            up.refs.decr();
        }
    }
}
impl<T: UniquePointee> AsRef<T> for UniquePointer<T> {
    fn as_ref(&self) -> &T {
        if self.is_null() {
            panic!("null pointer: {:#?}", self);
        }
        self.inner_ref()
    }
}
impl<T: UniquePointee> AsMut<T> for UniquePointer<T> {
    fn as_mut(&mut self) -> &mut T {
        if self.is_null() {
            panic!("null pointer: {:#?}", self);
        }
        self.inner_mut()
    }
}

impl<T: UniquePointee> Deref for UniquePointer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner_ref()
    }
}

impl<T: UniquePointee> DerefMut for UniquePointer<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner_mut()
    }
}

impl<T: UniquePointee> Drop for UniquePointer<T>
where
    T: Debug,
{
    fn drop(&mut self) {
        self.drop_in_place();
    }
}

impl<T: UniquePointee> From<&T> for UniquePointer<T>
where
    T: Debug,
{
    fn from(data: &T) -> UniquePointer<T> {
        UniquePointer::<T>::from_ref(data)
    }
}
impl<T: UniquePointee> From<&mut T> for UniquePointer<T>
where
    T: Debug,
{
    fn from(data: &mut T) -> UniquePointer<T> {
        UniquePointer::<T>::from_ref_mut(data)
    }
}
impl<T: UniquePointee> From<T> for UniquePointer<T>
where
    T: Debug,
{
    fn from(data: T) -> UniquePointer<T> {
        UniquePointer::from_ref(&data)
    }
}
/// The [`Clone`] implementation of `UniquePointer` is special
/// because it flags cloned values as clones such that a double-free
/// doesn not occur.
impl<T: UniquePointee> Clone for UniquePointer<T>
where
    T: Debug,
{
    fn clone(&self) -> UniquePointer<T> {
        self.incr_ref();
        let mut clone = UniquePointer::<T>::copy();
        clone.set_mut_ptr(self.mut_ptr, false);
        clone.refs = self.refs.clone();
        clone.alloc = self.alloc;
        clone.written = self.written;
        clone
    }
}

impl<T: UniquePointee> Pointer for UniquePointer<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:016x}", self.addr())
    }
}

impl<T: UniquePointee> Debug for UniquePointer<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "UniquePointer{}",
            [
                format!("{:016x}", self.addr()),
                if self.is_not_null() {
                    [
                        format!("[src={:#?}]", self.inner_ref()),
                        format!("[refs={}]", self.refs),
                    ]
                    .join("")
                } else {
                    [
                        format!("[refs={}]", self.refs),
                        format!("[alloc={}]", self.alloc),
                        format!("[written={}]", self.written),
                    ]
                    .join("")
                },
                format!("[is_copy={}]", self.is_copy),
            ]
            .join("")
        )
    }
}

impl<T: UniquePointee + PartialEq> PartialEq<UniquePointer<T>> for UniquePointer<T> {
    fn eq(&self, fles: &UniquePointer<T>) -> bool {
        if self.addr() == fles.addr() {
            return true;
        }
        if self.is_null() {
            let eq = fles.is_null();
            return eq;
        }
        self.inner_ref().eq(fles.inner_ref())
    }
}
impl<T: UniquePointee + Eq> Eq for UniquePointer<T> {}
impl<T: UniquePointee + PartialOrd> PartialOrd<UniquePointer<T>> for UniquePointer<T> {
    fn partial_cmp(&self, other: &UniquePointer<T>) -> Option<Ordering> {
        if self.is_null() {
            return None;
        }
        if self.addr() == other.addr() {
            return Some(Ordering::Equal);
        }
        self.inner_ref().partial_cmp(other.inner_ref())
    }
}

impl<T: UniquePointee + PartialOrd> PartialOrd<T> for UniquePointer<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.is_null() {
            return None;
        }
        self.inner_ref().partial_cmp(other)
    }
}
impl<T: UniquePointee + PartialEq> PartialEq<T> for UniquePointer<T> {
    fn eq(&self, other: &T) -> bool {
        if self.is_null() {
            return false;
        }
        self.inner_ref().eq(other)
    }
}

impl<T: UniquePointee + Ord> Ord for UniquePointer<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_null() {
            return Ordering::Less;
        }
        self.inner_ref().cmp(other.inner_ref())
    }
}

impl<T: UniquePointee + Hash> Hash for UniquePointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner_ref().hash(state)
    }
}

// impl<T: Deref, S: Deref> PartialEq<&UniquePointer<S>> for UniquePointer<T>
// where
//     T: PartialEq<S::Target> + UniquePointee,
//     S: UniquePointee,
// {
//     fn eq(&self, other: &&UniquePointer<S>) -> bool {
//         T::eq(self, other)
//     }

//     fn ne(&self, other: &&UniquePointer<S>) -> bool {
//         T::ne(self, other)
//     }
// }

// impl<T: Deref, S: Deref> PartialEq<UniquePointer<S>> for UniquePointer<T>
// where
//     T: PartialEq<S::Target> + UniquePointee,
//     S: UniquePointee,
// {
//     fn eq(&self, other: &UniquePointer<S>) -> bool {
//         T::eq(self, other)
//     }

//     fn ne(&self, other: &UniquePointer<S>) -> bool {
//         T::ne(self, other)
//     }
// }

// impl<T: Deref<Target: Eq> + Eq + PartialEq<<T as Deref>::Target>> Eq for UniquePointer<T> where
//     T: UniquePointee
// {
// }

// impl<T: Deref, S: Deref> PartialOrd<UniquePointer<S>> for UniquePointer<T>
// where
//     T: PartialOrd<S::Target> + UniquePointee,
//     S: UniquePointee,
// {
//     fn partial_cmp(&self, other: &UniquePointer<S>) -> Option<Ordering> {
//         T::partial_cmp(self, other)
//     }

//     fn lt(&self, other: &UniquePointer<S>) -> bool {
//         T::lt(self, other)
//     }

//     fn le(&self, other: &UniquePointer<S>) -> bool {
//         T::le(self, other)
//     }

//     fn gt(&self, other: &UniquePointer<S>) -> bool {
//         T::gt(self, other)
//     }

//     fn ge(&self, other: &UniquePointer<S>) -> bool {
//         T::ge(self, other)
//     }
// }

// impl<T: Deref<Target: Ord> + Ord + PartialOrd<<T as Deref>::Target>> Ord for UniquePointer<T>
// where
//     T: UniquePointee,
// {
//     fn cmp(&self, other: &Self) -> Ordering {
//         T::cmp(self, other)
//     }
// }

// impl<T: Deref<Target: Hash> + Hash> Hash for UniquePointer<T>
// where
//     T: UniquePointee,
// {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         T::hash(self, state);
//     }
// }
