use crate::{Pointee, RefCounter};
use std::alloc::Layout;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::convert::{AsMut, AsRef};
use std::fmt::{Debug, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

pub const ISACOPY: u8 = 0b0001;
pub const ISALLOC: u8 = 0b0010;
pub const WRITTEN: u8 = 0b0100;

/// `UniquePointer` is an experimental data structure that makes
/// extensive use of unsafe rust to provide a shared pointer
/// throughout the runtime of a rust program as transparently as
/// possible.
///
/// [UniquePointer](Self)'s design's purpose is two-fold:
///
/// - Leverage the implementation of circular data structures such as
/// Lisp cons cells.
///
/// - Making easier the task of practicing the implementation of basic
/// computer science data-structures (e.g.: Binary Trees, Linked Lists
/// etc) such that the concept of pointer is as close to C as possible
/// in terms of developer experience and so when a CS teacher speaks
/// in terms of pointers, students can use [UniquePointer](Self) in their
/// data-structures knowing that cloning their data-structures also
/// means cloning the pointers transparently.
///
/// In fact, the [author](https://github.com/gabrielfalcao/) designed
/// `UniquePointer` while studying the MIT CourseWare material of
/// professor [Erik Demaine](https://github.com/edemaine) as well as
/// to implement lisp [cons](https://en.wikipedia.org/wiki/Cons) cells
/// and ring-buffers.
///
/// To this point the author reiterates: `UniquePointer` is an
/// **experimental** data-structure designed primarily as a
/// building-block of other data-structures in rust.
///
/// `UniquePointer` provides the methods [`UniquePointer::cast_mut`]
/// and [`UniquePointer::cast_const`] not unlike those of raw
/// pointers, and also implements the methods
/// [`UniquePointer::as_ref`] and [`UniquePointer::as_mut`] with a
/// signature compatible with that of the [AsRef] and [AsMut]
/// traits such that users of raw pointers can migrate to
/// [UniquePointer](Self) without much difficulty.
///
/// `UniquePointer` is designed a way such that Enums and Structs
/// using `UniquePointer` can safely clone `UniquePointer` while the
/// memory address and provenance of its value is shared.
///
/// [UniquePointer](Self) is able to extend lifetimes because it maintains
/// its own reference counting outside of the rust compiler.
///
/// Reference Counting is provided by [RefCounter] which uses unsafe
/// rust to ensure that ref counts are shared across cloned objects
/// memory.
///
/// Both [UniquePointer](Self) and [RefCounter] use relatively obscure
/// rust techniques under the hood to allow writing in non-mut
/// references in strategic occasions such as incrementing its
/// reference count within its [Clone] implementation.
///
/// UniquePointer only supports [Sized] types, that is, [Zero-Sized
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
/// # Caveats
///
/// - Only supports types that implement [Debug]
/// - Does not support [ZSTs](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts) (Zero-Sized Types)
/// - [UniquePointer](Self) **IS NOT THREAD SAFE**
///
/// # Lisp Cons Cell Example
///
/// ```
/// use std::iter::Extend;
///
/// use unique_pointer::{RefCounter, UniquePointer};
/// # use std::borrow::Cow;
/// # use std::convert::{AsMut, AsRef};
/// #
/// # #[derive(Clone, PartialOrd, Ord, Default, PartialEq, Eq, Hash)]
/// # pub enum Value<'c> {
/// #     #[default]
/// #     Nil,
/// #     String(Cow<'c, str>),
/// #     Byte(u8),
/// #     UInt(u64),
/// #     Int(i64),
/// # }
/// # impl<'c> Value<'_> {
/// #     pub fn nil() -> Value<'c> {
/// #         Value::Nil
/// #     }
/// #
/// #     pub fn is_nil(&self) -> bool {
/// #         if *self == Value::Nil {
/// #             true
/// #         } else {
/// #             false
/// #         }
/// #     }
/// # }
/// #
/// # impl<'c> Drop for Value<'c> {
/// #     fn drop(&mut self) {}
/// # }
/// #
/// # impl std::fmt::Display for Value<'_> {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
/// #         write!(
/// #             f,
/// #             "{}",
/// #             match self {
/// #                 Value::Nil => "nil".to_string(),
/// #                 Value::String(h) => format!("{}", h),
/// #                 Value::Byte(h) => format!("{}", h),
/// #                 Value::UInt(h) => format!("{}", h),
/// #                 Value::Int(h) => format!("{}", h),
/// #             }
/// #         )
/// #     }
/// # }
/// # impl std::fmt::Debug for Value<'_> {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
/// #         write!(
/// #             f,
/// #             "{}",
/// #             match self {
/// #                 Value::Nil => "nil".to_string(),
/// #                 Value::String(h) => format!("{:#?}", h),
/// #                 Value::Byte(h) => format!("{}u8", h),
/// #                 Value::UInt(h) => format!("{}u64", h),
/// #                 Value::Int(h) => format!("{}i64", h),
/// #             }
/// #         )
/// #     }
/// # }
/// #
/// # impl<'c> From<u8> for Value<'c> {
/// #     fn from(value: u8) -> Value<'c> {
/// #         Value::Byte(value)
/// #     }
/// # }
/// # impl<'c> From<u64> for Value<'c> {
/// #     fn from(value: u64) -> Value<'c> {
/// #         Value::UInt(value)
/// #     }
/// # }
/// # impl<'c> From<i64> for Value<'c> {
/// #     fn from(value: i64) -> Value<'c> {
/// #         Value::Int(value)
/// #     }
/// # }
/// # impl<'c> From<&'c str> for Value<'c> {
/// #     fn from(value: &'c str) -> Value<'c> {
/// #         Value::String(Cow::from(value))
/// #     }
/// # }
/// # impl<'c> From<Cow<'c, str>> for Value<'c> {
/// #     fn from(value: Cow<'c, str>) -> Value<'c> {
/// #         Value::from(value.into_owned())
/// #     }
/// # }
/// # impl<'c> From<&'c mut str> for Value<'c> {
/// #     fn from(value: &'c mut str) -> Value<'c> {
/// #         Value::String(Cow::<'c, str>::Borrowed(&*value))
/// #     }
/// # }
/// # impl<'c> From<String> for Value<'c> {
/// #     fn from(value: String) -> Value<'c> {
/// #         Value::String(Cow::from(value))
/// #     }
/// # }
/// # impl<'c> From<Option<String>> for Value<'c> {
/// #     fn from(value: Option<String>) -> Value<'c> {
/// #         match value {
/// #             None => Value::Nil,
/// #             Some(value) => Value::from(value),
/// #         }
/// #     }
/// # }
/// #
/// # impl<'c> AsRef<Value<'c>> for Value<'c> {
/// #     fn as_ref(&self) -> &Value<'c> {
/// #         unsafe { &*self }
/// #     }
/// # }
/// # impl<'c> AsMut<Value<'c>> for Value<'c> {
/// #     fn as_mut(&mut self) -> &mut Value<'c> {
/// #         unsafe { &mut *self }
/// #     }
/// # }
/// #
/// # impl<'c> PartialEq<&Value<'c>> for Value<'c> {
/// #     fn eq(&self, other: &&Value<'c>) -> bool {
/// #         let other = unsafe { &**other };
/// #         self == other
/// #     }
/// # }
/// #
/// # impl<'c> PartialEq<&mut Value<'c>> for Value<'c> {
/// #     fn eq(&self, other: &&mut Value<'c>) -> bool {
/// #         let other = unsafe { &**other };
/// #         self == other
/// #     }
/// # }
/// #
///
/// #[derive(Debug, Hash)]
/// pub struct Cell<'c> {
///     head: UniquePointer<Value<'c>>,
///     tail: UniquePointer<Cell<'c>>,
///     refs: RefCounter,
///     length: usize,
/// }
///
/// impl<'c> Cell<'c> {
///     pub fn nil() -> Cell<'c> {
///         Cell {
///             head: UniquePointer::<Value<'c>>::null(),
///             tail: UniquePointer::<Cell<'c>>::null(),
///             refs: RefCounter::null(),
///             length: 0,
///         }
///     }
///
///     pub fn is_nil(&self) -> bool {
///         self.head.is_null() && self.tail.is_null()
///     }
///
///     pub fn new(value: Value<'c>) -> Cell<'c> {
///         let mut cell = Cell::nil();
///         cell.write(value);
///         cell
///     }
///
///     fn write(&mut self, value: Value<'c>) {
///         self.head.write(value);
///         self.refs.write(1);
///         self.length = 1;
///     }
///
///     fn swap_head(&mut self, other: &mut Self) {
///         self.head = unsafe {
///             let head = other.head.propagate();
///             other.head = self.head.propagate();
///             head
///         };
///     }
///
///     fn swap_refs(&mut self, other: &mut Self) {
///         self.refs = {
///             let refs = other.refs.clone();
///             other.refs = self.refs.clone();
///             refs
///         };
///     }
///
///     pub fn head(&self) -> Option<Value<'c>> {
///         self.head.try_read()
///     }
///
///     pub fn add(&mut self, new: &mut Cell<'c>) {
///         new.incr_ref();
///         self.incr_ref();
///         if self.head.is_null() {
///             unsafe {
///                 if !new.head.is_null() {
///                     self.swap_head(new);
///                 }
///
///                 if !new.tail.is_null() {
///                     let tail = new.tail.inner_mut();
///                     tail.swap_head(new);
///                     self.swap_refs(new);
///                 }
///                 self.tail = UniquePointer::read_only(new);
///             }
///         } else {
///             if self.tail.is_null() {
///                 self.tail = UniquePointer::read_only(new);
///             } else {
///                 self.tail.inner_mut().add(new);
///             }
///         }
///     }
///
///     pub fn pop(&mut self) -> bool {
///         if !self.tail.is_null() {
///             self.tail.drop_in_place();
///             self.tail = UniquePointer::null();
///             true
///         } else if !self.head.is_null() {
///             self.head.drop_in_place();
///             self.head = UniquePointer::null();
///             true
///         } else {
///             false
///         }
///     }
///
///     pub fn is_empty(&self) -> bool {
///         self.len() > 0
///     }
///
///     pub fn len(&self) -> usize {
///         let mut len = 0;
///         if !self.head.is_null() {
///             len += 1
///         }
///         if let Some(tail) = self.tail() {
///             len += tail.len();
///         }
///         len
///     }
///
///     pub fn tail(&self) -> Option<&'c Cell<'c>> {
///         self.tail.as_ref()
///     }
///
///     pub fn values(&self) -> Vec<Value<'c>> {
///         let mut values = Vec::<Value>::new();
///         if let Some(head) = self.head() {
///             values.push(head.clone());
///         }
///         if let Some(tail) = self.tail() {
///             values.extend(tail.values());
///         }
///         values
///     }
///
///     fn incr_ref(&mut self) {
///         self.refs += 1;
///         if !self.tail.is_null() {
///             if let Some(tail) = self.tail.as_mut() {
///                 tail.incr_ref();
///             }
///         }
///     }
///
///     fn decr_ref(&mut self) {
///         self.refs -= 1;
///         if !self.tail.is_null() {
///             if let Some(tail) = self.tail.as_mut() {
///                 tail.decr_ref();
///             }
///         }
///     }
///
///     fn dealloc(&mut self) {
///         if self.refs > 0 {
///             self.decr_ref();
///         } else {
///             self.head.drop_in_place();
///             self.tail.drop_in_place();
///         }
///     }
/// }
///
/// impl<'c> From<Value<'c>> for Cell<'c> {
///     fn from(value: Value<'c>) -> Cell<'c> {
///         Cell::new(value)
///     }
/// }
/// impl<'c> From<&'c str> for Cell<'c> {
///     fn from(value: &'c str) -> Cell<'c> {
///         let value = Value::from(value);
///         Cell::new(value)
///     }
/// }
/// impl<'c> From<u8> for Cell<'c> {
///     fn from(value: u8) -> Cell<'c> {
///         Cell::new(Value::Byte(value))
///     }
/// }
/// impl<'c> From<u64> for Cell<'c> {
///     fn from(value: u64) -> Cell<'c> {
///         if value < u8::MAX.into() {
///             Cell::new(Value::Byte(value as u8))
///         } else {
///             Cell::new(Value::UInt(value))
///         }
///     }
/// }
/// impl<'c> From<i32> for Cell<'c> {
///     fn from(value: i32) -> Cell<'c> {
///         if let Ok(value) = TryInto::<u64>::try_into(value) {
///             Cell::new(Value::UInt(value))
///         } else {
///             Cell::new(Value::Int(value.into()))
///         }
///     }
/// }
/// impl<'c> From<i64> for Cell<'c> {
///     fn from(value: i64) -> Cell<'c> {
///         Cell::new(Value::from(value))
///     }
/// }
///
/// impl<'c> PartialEq<Cell<'c>> for Cell<'c> {
///     fn eq(&self, other: &Cell<'c>) -> bool {
///         if self.head.is_null() == other.head.is_null() {
///             true
///         } else if let Some(head) = self.head() {
///             if let Some(value) = other.head() {
///                 return head == value && (self.tail() == other.tail());
///             } else {
///                 false
///             }
///         } else {
///             false
///         }
///     }
/// }
///
/// impl<'c> Default for Cell<'c> {
///     fn default() -> Cell<'c> {
///         Cell::nil()
///     }
/// }
/// impl<'c> Clone for Cell<'c> {
///     fn clone(&self) -> Cell<'c> {
///         let mut cell = Cell::nil();
///         cell.refs = self.refs.clone();
///         if self.head.is_not_null() {
///             cell.head = self.head.clone();
///         }
///         if self.tail.is_not_null() {
///             cell.tail = self.tail.clone();
///         }
///         cell
///     }
/// }
/// impl<'c> Drop for Cell<'c> {
///     fn drop(&mut self) {
///         self.dealloc();
///     }
/// }
/// ```
///
#[doc(alias = "Pointer")]
pub struct UniquePointer<T: Pointee> {
    mut_addr: usize,
    mut_ptr: *mut T,
    refs: RefCounter,
    flags: u8,
}
impl<'c, T: Pointee + 'c> UniquePointer<T> {
    /// creates a NULL `UniquePointer` ready to be written via [write].
    pub fn null() -> UniquePointer<T> {
        UniquePointer {
            mut_addr: 0,
            mut_ptr: std::ptr::null_mut::<T>(),
            refs: RefCounter::new(),
            flags: 0,
        }
    }

    /// creates a new `UniquePointer` by effectively
    /// reading the value referenced by **`src`**
    ///
    pub fn from_ref(src: &T) -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.write_ref(src);
        up
    }

    /// `from_ref_mut` creates a new `UniquePointer` by effectively
    /// reading the value referenced by **`src`**
    ///
    pub fn from_ref_mut(src: &mut T) -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.write_ref_mut(src);
        up
    }

    /// is designed for use within the [Clone] implementation
    /// of `UniquePointer`.
    ///
    /// The [copy] method creates a NULL `UniquePointer` flagged as
    /// [`is_copy`] such that a double-free does not happen in
    /// [dealloc].
    fn copy() -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.flags = up.flags | (ISACOPY);
        up
    }

    /// produces a copy of a `UniquePointer` which is not a copy in
    /// the sense that [`UniquePointer::is_copy`] returns true.
    ///
    /// Because of that rationale a double-free occurs if there are
    /// two or more "containers" (e.g.: [struct](std#keyword.struct.html)s and [enum](std#keyword.enum.html)s)
    /// implementing [Drop] and holding the same propagated
    /// `UniquePointer` instance. For this reason
    /// [`UniquePointer::propagate`] is unsafe.
    ///
    /// [`UniquePointer::propagate`] can be relatively observed as a
    /// drop-in replacement to [`UniquePointer::clone`] for cases
    /// when, for instance, swapping `UniquePointer` "instances"
    /// between instances of `UniquePointer`-containing (structs
    /// and/or enums) is desired.
    ///
    /// Example
    ///
    /// ```
    /// use unique_pointer::UniquePointer;
    /// use std::fmt::Debug;
    /// use std::cmp::PartialEq;
    ///
    /// #[derive(Clone, Debug, Hash)]
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
        back_node.flags = self.flags;
        back_node
    }
    /// `unlock_reference` extends the lifetime of `&T` to `&'t T` and
    /// unlocks `&'t T` into a `&'t mut T`
    ///
    /// This function is primarily designed to permit data-structures
    /// implementing their own reference counting [`Clone`] to "break
    /// out" of a read-only reference, so to speak, so that its
    /// references can be incremented.
    ///
    /// Example
    ///
    /// ```
    /// use std::fmt::Debug;
    /// use unique_pointer::{RefCounter, UniquePointer};
    ///
    /// #[derive(Debug, Hash)]
    /// pub struct LinkedList<T: Debug + Clone> {
    ///     pub item: T,
    ///     pub next: UniquePointer<LinkedList<T>>,
    ///     pub refs: usize,
    /// }
    /// impl<T: Debug + Clone> LinkedList<T> {
    ///     pub fn new(item: T) -> LinkedList<T> {
    ///         LinkedList {
    ///             item,
    ///             next: UniquePointer::null(),
    ///             refs: 1,
    ///         }
    ///     }
    ///     pub fn item(&self) -> T {
    ///         self.item.clone()
    ///     }
    ///     fn incr_ref(&mut self) {
    ///         self.refs += 1;
    ///     }
    ///     fn decr_ref(&mut self) {
    ///         if self.refs > 0 {
    ///             self.refs -= 1;
    ///         }
    ///     }
    ///     fn dealloc(&mut self) {
    ///         self.decr_ref();
    ///         if self.next.is_not_null() {
    ///             self.next.inner_mut().dealloc()
    ///         }
    ///         if self.refs == 0 {
    ///             self.next.drop_in_place();
    ///         }
    ///     }
    ///     pub fn append(&mut self, value: T) -> LinkedList<T> {
    ///         let next = LinkedList::new(value);
    ///         self.next.write_ref(&next);
    ///         next
    ///     }
    ///
    ///     pub fn next(&self) -> Option<&LinkedList<T>> {
    ///         self.next.as_ref()
    ///     }
    ///
    ///     pub fn len(&self) -> usize {
    ///         let mut length = 1;
    ///
    ///         if let Some(next) = self.next() {
    ///             length += 1;
    ///             length += next.len();
    ///         }
    ///         length
    ///     }
    /// }
    /// impl<T: Debug + Clone> Clone for LinkedList<T> {
    ///     fn clone(&self) -> LinkedList<T> {
    ///         unsafe {
    ///             UniquePointer::<LinkedList<T>>::unlock_reference(self).incr_ref();
    ///         }
    ///         let mut list = LinkedList::new(self.item());
    ///         list.refs = self.refs;
    ///         list.next = self.next.clone();
    ///         list
    ///     }
    /// }
    /// impl<T: Debug + Clone> Drop for LinkedList<T> {
    ///     fn drop(&mut self) {
    ///         self.dealloc();
    ///     }
    /// }
    /// let mut a = LinkedList::new("a");
    /// let mut b = a.append("b");
    /// b.append("c");
    ///
    /// assert_eq!(a.refs, 1);
    /// assert_eq!(a.len(), 3);
    /// let z = a.clone();
    /// assert_eq!(z.len(), 3);
    /// assert_eq!(a.refs, 2);
    /// assert_eq!(z.refs, 2);
    /// ```
    #[allow(mutable_transmutes)]
    pub unsafe fn unlock_reference<'t>(read_only: &T) -> &'t mut T {
        let extended = unsafe { std::mem::transmute::<&T, &'t T>(read_only) };
        let unlocked = unsafe { std::mem::transmute::<&'t T, &'t mut T>(extended) };
        unlocked
    }
    /// calls [`UniquePointer::copy_from_ref`] to create a *read-only* `UniquePointer` from a
    /// reference of `T`, useful for iterating over self-referential
    /// data structures.
    ///
    /// Example:
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
    pub fn read_only(data: &T) -> UniquePointer<T> {
        UniquePointer::copy_from_ref(data, 1)
    }

    /// calls [`UniquePointer::copy_from_mut_ptr`] to create a *read-only*
    /// `UniquePointer` from a reference of `T`, useful for
    /// iterating over self-referential data structures that use
    /// [RefCounter] to count refs.
    ///
    /// Note: [`UniquePointer::read_only`] might be a better alternative when `T` is
    /// a data structure that does not use [RefCounter].
    pub fn copy_from_ref(data: &T, refs: usize) -> UniquePointer<T> {
        let ptr = (data as *const T).cast_mut();
        UniquePointer::copy_from_mut_ptr(ptr, refs)
    }

    /// creates a *read-only* `UniquePointer`
    /// from a reference of `T`, useful for iterating over
    /// self-referential data structures that use [RefCounter] to
    /// count refs.
    ///
    /// Note: [`UniquePointer::read_only`] might be a better alternative when `T` is
    /// a data structure that does not use [RefCounter].
    pub fn copy_from_mut_ptr(ptr: *mut T, refs: usize) -> UniquePointer<T> {
        let addr = UniquePointer::provenance_of_mut_ptr(ptr);
        let refs = RefCounter::from(refs);
        UniquePointer {
            mut_addr: addr,
            mut_ptr: ptr,
            refs: refs,
            flags: (ISACOPY | ISALLOC | WRITTEN),
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
        #[cfg(feature = "null-check")]
        if mut_is_null {
            assert!(self.mut_addr == 0);
        } else {
            assert!(self.mut_addr != 0);
        }
        let is_null = mut_is_null;
        is_null
    }

    /// returns true if the `UniquePointer` is not
    /// NULL. [`UniquePointer::is_not_null`] is a idiomatic shortcut
    /// to negating a call to [`UniquePointer::is_null`] such that the
    /// negation is less likely to be clearly visible.
    pub fn is_not_null(&self) -> bool {
        !self.is_null()
    }

    /// returns true if the `UniquePointer` is not a
    /// copy. [`UniquePointer::is_not_copy`] is a idiomatic shortcut
    /// to negating a call to [`UniquePointer::is_copy`] such that the
    /// negation is less likely to be clearly visible.
    pub fn is_not_copy(&self) -> bool {
        !self.is_copy()
    }

    /// returns true if the `UniquePointer` is not NULL
    /// and is not flagged as a copy, meaning it can be deallocated
    /// without concern for double-free.
    pub fn can_dealloc(&self) -> bool {
        ((self.flags & ISALLOC) == ISALLOC) && self.is_not_copy() && self.is_not_null()
    }

    /// returns true if the `UniquePointer` has been
    /// allocated and therefore is no longer a NULL pointer.
    pub fn is_allocated(&self) -> bool {
        let is_allocated = ((self.flags & ISALLOC) == ISALLOC) && self.is_not_null();
        is_allocated
    }

    /// returns true if the `UniquePointer` has been written to
    pub fn is_written(&self) -> bool {
        let is_written = ((self.flags & WRITTEN) == WRITTEN) && self.is_allocated();
        is_written
    }

    /// returns true if a `UniquePointer` is a "copy" of
    /// another `UniquePointer` in the sense that dropping or
    /// "hard-deallocating" said `UniquePointer` does not incur a
    /// double-free.
    pub fn is_copy(&self) -> bool {
        ((self.flags & ISACOPY) == ISACOPY)
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
        self.flags |= ISALLOC;
    }

    /// compatibility API to a raw mut pointer's [`pointer::cast_mut`].
    pub fn cast_mut(&self) -> *mut T {
        if self.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        } else {
            self.mut_ptr
        }
    }

    /// compatibility API to a raw const pointer's [`pointer::cast_const`].
    pub fn cast_const(&self) -> *const T {
        if self.is_null() {
            panic!("NULL POINTER: {:#?}", self);
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

        self.flags |= (WRITTEN);
    }

    /// takes a mutable reference to a value and
    /// writes to a `UniquePointer`
    pub fn write_ref_mut(&mut self, data: &mut T) {
        self.alloc();
        unsafe {
            let ptr = data as *mut T;
            ptr.copy_to(self.mut_ptr, 1);
        };
        self.flags |= (WRITTEN);
    }

    /// takes a read-only reference to a value and
    /// writes to a `UniquePointer`
    pub fn write_ref(&mut self, data: &T) {
        self.alloc();
        unsafe {
            let ptr = data as *const T;
            ptr.copy_to(self.mut_ptr, 1);
        };
        self.flags |= (WRITTEN);
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
        if self.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        if !self.is_written() {
            panic!("{:#?} not written", self);
        }
        let ptr = self.cast_const();
        unsafe { ptr.read() }
    }

    /// reads data from memory `UniquePointer`
    pub fn try_read(&self) -> Option<T> {
        if self.is_null() {
            return None;
        }
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

    /// compatibility layer to [`std::pointer::as_mut`](std#primitive.pointer.html#formatting-parameters)
    pub fn as_mut(&mut self) -> Option<&'c mut T> {
        if self.is_written() {
            Some(self.inner_mut())
        } else {
            None
        }
    }

    /// Returns a `Box<T>` without dropping T, panics if
    /// [UniquePointer](Self) points to null.
    ///
    /// See [into_box](Self::into_box) for a version that returns
    /// [`Option<Box<T>>`].
    ///
    /// Example boxing a type that does not implement Clone
    ///
    /// ```
    /// use unique_pointer::UniquePointer;
    /// use std::collections::BTreeMap;
    /// use std::fmt::{Display, Debug, Formatter};
    ///
    /// pub trait Matcher {
    ///     fn to_str(&self) -> String;
    ///     fn to_dbg(&self) -> String {
    ///         format!("{:#?}", self.to_str())
    ///     }
    /// }
    ///
    /// impl Debug for dyn Matcher {
    ///     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    ///         write!(f, "{}", self.to_str())
    ///     }
    /// }
    ///
    /// #[derive(Debug)]
    /// pub enum Match {
    ///     Literal(String),
    ///     Rule(Box<Rule>),
    ///     Matcher(Box<dyn Matcher>),
    ///     Sequence(Vec<Box<dyn Matcher>>),
    /// }
    ///
    /// pub(crate) static mut RULES: BTreeMap<&'static str, UniquePointer<Match>> = BTreeMap::new();
    ///
    /// #[allow(static_mut_refs)]
    /// pub(crate) fn register_match<T: Display>(string: T, r#match: Match) -> Match {
    ///     unsafe {
    ///         RULES.insert(string.to_string().leak(), UniquePointer::from_ref(&r#match));
    ///     }
    ///     r#match
    /// }
    /// #[derive(Debug)]
    /// pub struct Rule {
    ///     sym: String,
    ///     matcher: Match,
    /// }
    /// impl Rule {
    ///     pub fn new<S: ToString>(symbol: S, matcher: impl Into<Match>) -> Rule {
    ///         Rule {
    ///             sym: symbol.to_string(),
    ///             matcher: matcher.into(),
    ///         }
    ///     }
    ///     pub fn symbol(&self) -> &str {
    ///         self.sym.as_ref()
    ///     }
    /// }
    /// impl From<Rule> for Match {
    ///     fn from(rule: Rule) -> Match {
    ///         let rule = UniquePointer::from(rule);
    ///         let symbol = rule.inner_ref().symbol();
    ///         register_match(symbol, Match::Rule(rule.into_box_unchecked()))
    ///     }
    /// }
    /// ```
    pub fn into_box_unchecked(&self) -> Box<T> {
        if self.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        Box::new(self.read())
    }

    /// Returns a `Option<Box<T>>` without dropping T, returns `None`
    /// if pointing to null.
    ///
    /// See [into_box_unchecked](Self::into_box_unchecked) for a
    /// version that returns [`Box<T>`].
    pub fn into_box(&self) -> Option<Box<T>> {
        if self.is_null() {
            return None;
        }
        Some(self.into_box_unchecked())
    }

    /// deallocates a `UniquePointer`.
    ///
    /// The [soft] boolean argument indicates whether the
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
    /// When [ptr] is a NULL pointer and the internal pointer of
    /// `UniquePointer` in question is NOT NULL, then it is
    /// deallocated prior to setting it to NULL.
    fn set_mut_ptr(&mut self, ptr: *mut T, dealloc: bool) {
        if ptr.is_null() {
            if dealloc && self.is_allocated() {
                self.flags = 0;
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

    /// is internally used by [dealloc] when the number of
    /// references gets down to zero in a "soft" deallocation and
    /// immediately in a "hard" deallocation.
    ///
    /// See [dealloc] for more information regarding the difference
    /// between "soft" and "hard" deallocation.
    fn free(&mut self) {
        if !self.can_dealloc() {
            return;
        }
        if !self.is_null() {
            self.set_mut_ptr(std::ptr::null_mut::<T>(), false);
            self.refs.drain();
        }
        self.flags = 0;
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

impl<T: Pointee> UniquePointer<T> {
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
impl<'c, T: Pointee + 'c> UniquePointer<T> {
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
impl<T: Pointee> UniquePointer<T> {
    fn incr_ref(&self) {
        if self.is_null() {
            return;
        }
        self.refs.incr();
    }

    fn decr_ref(&self) {
        if self.refs == 0 {
            return;
        }
        self.refs.decr();
    }
}
impl<T: Pointee> AsRef<T> for UniquePointer<T> {
    fn as_ref(&self) -> &T {
        if self.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        self.inner_ref()
    }
}
impl<T: Pointee> AsMut<T> for UniquePointer<T> {
    fn as_mut(&mut self) -> &mut T {
        if self.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        self.inner_mut()
    }
}

impl<T: Pointee> Deref for UniquePointer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner_ref()
    }
}

impl<T: Pointee> DerefMut for UniquePointer<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner_mut()
    }
}

impl<T: Pointee> Drop for UniquePointer<T> {
    fn drop(&mut self) {
        self.drop_in_place();
    }
}

impl<T: Pointee> From<&T> for UniquePointer<T> {
    fn from(data: &T) -> UniquePointer<T> {
        UniquePointer::<T>::from_ref(data)
    }
}
impl<T: Pointee> From<&mut T> for UniquePointer<T> {
    fn from(data: &mut T) -> UniquePointer<T> {
        UniquePointer::<T>::from_ref_mut(data)
    }
}
impl<T: Pointee> From<T> for UniquePointer<T> {
    fn from(data: T) -> UniquePointer<T> {
        let mut up = UniquePointer::<T>::null();
        up.write(data);
        up
    }
}
/// The [Clone] implementation of `UniquePointer` is special
/// because it flags cloned values as clones such that a double-free
/// doesn not occur.
impl<T: Pointee> Clone for UniquePointer<T> {
    fn clone(&self) -> UniquePointer<T> {
        self.incr_ref();
        let mut clone = UniquePointer::<T>::copy();
        clone.set_mut_ptr(self.mut_ptr, false);
        clone.refs = self.refs.clone();
        clone.flags = self.flags;
        clone
    }
}

impl<T: Pointee> Pointer for UniquePointer<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:016x}", self.addr())
    }
}

impl<T: Pointee> Debug for UniquePointer<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "UniquePointer{}",
            [
                format!("{:016x}", self.addr()),
                if self.is_not_null() {
                    [
                        #[cfg(not(feature = "allow-no-debug"))]
                        format!("[src={:#?}]", self.inner_ref()),
                        #[cfg(feature = "allow-no-debug")]
                        format!("[src={:p}]", self.inner_ref()),
                        format!("[refs={}]", self.refs),
                    ]
                    .join("")
                } else {
                    [
                        format!("[refs={}]", self.refs),
                        format!("[alloc={}]", self.is_allocated()),
                        format!("[written={}]", self.is_written()),
                    ]
                    .join("")
                },
                format!("[is_copy={}]", self.is_copy()),
            ]
            .join("")
        )
    }
}

impl<T: Pointee + PartialEq> PartialEq<UniquePointer<T>> for UniquePointer<T> {
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
impl<T: Pointee + Eq> Eq for UniquePointer<T> {}
impl<T: Pointee + PartialOrd> PartialOrd<UniquePointer<T>> for UniquePointer<T> {
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

impl<T: Pointee + PartialOrd> PartialOrd<T> for UniquePointer<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.is_null() {
            return None;
        }
        self.inner_ref().partial_cmp(other)
    }
}
impl<T: Pointee + PartialEq> PartialEq<T> for UniquePointer<T> {
    fn eq(&self, other: &T) -> bool {
        if self.is_null() {
            return false;
        }
        self.inner_ref().eq(other)
    }
}

impl<T: Pointee + Ord> Ord for UniquePointer<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_null() {
            return Ordering::Less;
        }
        self.inner_ref().cmp(other.inner_ref())
    }
}

impl<T: Pointee> Hash for UniquePointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let size = std::mem::size_of::<T>();
        let mut ptr = self.mut_ptr as *mut u8;
        let bs = std::mem::size_of::<u8>();
        let end = unsafe { ptr.add(size) };
        while ptr < end {
            let mut byte = 0u8;
            unsafe {
                ptr.copy_to(&raw mut byte, bs);
            }
            byte.hash(state);
            ptr = unsafe { ptr.add(bs) };
        }
    }
}

// impl<T: Deref, S: Deref> PartialEq<&UniquePointer<S>> for UniquePointer<T>
// where
//     T: PartialEq<S::Target> + Pointee,
//     S: Pointee,
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
//     T: PartialEq<S::Target> + Pointee,
//     S: Pointee,
// {
//     fn eq(&self, other: &UniquePointer<S>) -> bool {
//         T::eq(self, other)
//     }

//     fn ne(&self, other: &UniquePointer<S>) -> bool {
//         T::ne(self, other)
//     }
// }

// impl<T: Deref<Target: Eq> + Eq + PartialEq<<T as Deref>::Target>> Eq for UniquePointer<T> where
//     T: Pointee
// {
// }

// impl<T: Deref, S: Deref> PartialOrd<UniquePointer<S>> for UniquePointer<T>
// where
//     T: PartialOrd<S::Target> + Pointee,
//     S: Pointee,
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
//     T: Pointee,
// {
//     fn cmp(&self, other: &Self) -> Ordering {
//         T::cmp(self, other)
//     }
// }

// impl<T: Deref<Target: Hash> + Hash> Hash for UniquePointer<T>
// where
//     T: Pointee,
// {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         T::hash(self, state);
//     }
// }
