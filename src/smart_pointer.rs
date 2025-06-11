use std::alloc::Layout;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::convert::{AsMut, AsRef};
use std::fmt::{Debug, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use crate::Pointee;

#[doc(alias = "Pointer")]
pub struct SmartPointer<T: Pointee> {
    mut_addr: usize,
    mut_ptr: *mut T,
    written: bool,
}

impl<'c, T: Pointee + 'c> SmartPointer<T> {
    pub fn null() -> SmartPointer<T> {
        SmartPointer {
            mut_addr: 0,
            mut_ptr: std::ptr::null_mut::<T>(),
            written: false,
        }
    }

    pub fn from_ref(src: &T) -> SmartPointer<T> {
        let mut up = SmartPointer::<T>::null();
        up.write_ref(src);
        up
    }

    pub fn from_ref_mut(src: &mut T) -> SmartPointer<T> {
        let mut up = SmartPointer::<T>::null();
        up.write_ref_mut(src);
        up
    }

    pub fn copy(&self) -> SmartPointer<T> {
        let mut back_node = SmartPointer::<T>::null();
        back_node.set_mut_ptr(self.mut_ptr, false);
        back_node.written = self.written;
        back_node
    }

    pub fn copy_from_ref(data: &T) -> SmartPointer<T> {
        let ptr = (data as *const T).cast_mut();
        SmartPointer::copy_from_mut_ptr(ptr)
    }

    pub fn copy_from_mut_ptr(ptr: *mut T) -> SmartPointer<T> {
        let addr = SmartPointer::provenance_of_mut_ptr(ptr);
        SmartPointer {
            mut_addr: addr,
            mut_ptr: ptr,
            written: true,
        }
    }

    pub fn addr(&self) -> usize {
        self.mut_addr
    }

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

    pub fn is_not_null(&self) -> bool {
        !self.is_null()
    }

    pub fn is_allocated(&self) -> bool {
        let is_allocated = self.is_not_null() && self.is_written();
        is_allocated
    }

    pub fn is_written(&self) -> bool {
        let is_written = self.is_not_null() && self.written;
        is_written
    }

    fn alloc(&mut self) {
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
    }

    pub fn cast_mut(&self) -> *mut T {
        if self.is_null() {
            panic!("{:#?}", self);
        } else {
            self.mut_ptr
        }
    }

    pub fn cast_const(&self) -> *const T {
        if self.is_null() {
            panic!("{:#?}", self);
        } else {
            self.mut_ptr.cast_const()
        }
    }

    pub fn write(&mut self, data: T) {
        self.alloc();

        unsafe {
            self.mut_ptr.write(data);
        }

        self.written = true;
    }

    pub fn write_ref_mut(&mut self, data: &mut T) {
        self.alloc();
        unsafe {
            let ptr = data as *mut T;
            ptr.copy_to(self.mut_ptr, 1);
        };
        self.written = true;
    }

    pub fn write_ref(&mut self, data: &T) {
        self.alloc();
        unsafe {
            let ptr = data as *const T;
            ptr.copy_to(self.mut_ptr, 1);
        };
        self.written = true;
    }

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

    pub fn read(&self) -> T {
        if !self.is_written() {
            panic!("{:#?} not written", self);
        }
        let ptr = self.cast_const();
        unsafe { ptr.read() }
    }

    pub fn try_read(&self) -> Option<T> {
        if self.is_written() {
            Some(self.read())
        } else {
            None
        }
    }

    pub fn inner_ref(&self) -> &'c T {
        if self.mut_ptr.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        unsafe { std::mem::transmute::<&T, &'c T>(&*self.cast_const()) }
    }

    pub fn inner_mut(&mut self) -> &'c mut T {
        if self.mut_ptr.is_null() {
            panic!("NULL POINTER: {:#?}", self);
        }
        unsafe { std::mem::transmute::<&mut T, &'c mut T>(&mut *self.mut_ptr) }
    }

    pub fn as_ref(&self) -> Option<&'c T> {
        if self.is_written() {
            Some(self.inner_ref())
        } else {
            None
        }
    }

    pub fn as_mut(&mut self) -> Option<&'c mut T> {
        if self.is_written() {
            Some(self.inner_mut())
        } else {
            None
        }
    }

    fn set_mut_ptr(&mut self, ptr: *mut T, dealloc: bool) {
        if ptr.is_null() {
            self.set_mut_addr(0);
        } else {
            self.set_mut_addr(SmartPointer::<T>::provenance_of_mut_ptr(ptr));
        }
        self.mut_ptr = ptr;
    }

    fn set_mut_addr(&mut self, addr: usize) {
        self.mut_addr = addr;
    }

    pub fn extend_lifetime<'t>(&self) -> &'t T {
        unsafe { std::mem::transmute::<&T, &'t T>(self.inner_ref()) }
    }

    pub fn extend_lifetime_mut<'t>(&mut self) -> &'t mut T {
        unsafe { std::mem::transmute::<&mut T, &'t mut T>(self.inner_mut()) }
    }
}

impl<T: Pointee> SmartPointer<T> {
    pub fn provenance_of_const_ptr(ptr: *const T) -> usize {
        ptr.expose_provenance()
    }

    pub fn provenance_of_mut_ptr(ptr: *mut T) -> usize {
        ptr.expose_provenance()
    }

    pub fn provenance_of_ref(ptr: &T) -> usize {
        (&raw const ptr).expose_provenance()
    }

    pub fn provenance_of_mut(mut ptr: &mut T) -> usize {
        (&raw mut ptr).expose_provenance()
    }
}

#[allow(unused)]
impl<'c, T: Pointee + 'c> SmartPointer<T> {
    unsafe fn meta_mut(&'c self) -> &'c mut SmartPointer<T> {
        unsafe {
            let ptr = self.meta_mut_ptr();
            let up = &mut *ptr;
            std::mem::transmute::<&mut SmartPointer<T>, &'c mut SmartPointer<T>>(up)
        }
    }

    unsafe fn meta_mut_ptr(&self) -> *mut SmartPointer<T> {
        let ptr = self as *const SmartPointer<T>;
        unsafe {
            let ptr: *mut SmartPointer<T> =
                std::mem::transmute::<*const SmartPointer<T>, *mut SmartPointer<T>>(ptr);
            ptr
        }
    }
}
impl<T: Pointee> AsRef<T> for SmartPointer<T> {
    fn as_ref(&self) -> &T {
        if self.is_null() {
            panic!("null pointer: {:#?}", self);
        }
        self.inner_ref()
    }
}
impl<T: Pointee> AsMut<T> for SmartPointer<T> {
    fn as_mut(&mut self) -> &mut T {
        if self.is_null() {
            panic!("null pointer: {:#?}", self);
        }
        self.inner_mut()
    }
}

impl<T: Pointee> Deref for SmartPointer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner_ref()
    }
}

impl<T: Pointee> DerefMut for SmartPointer<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner_mut()
    }
}

impl<T: Pointee> From<&T> for SmartPointer<T>
where
    T: Debug,
{
    fn from(data: &T) -> SmartPointer<T> {
        SmartPointer::<T>::from_ref(data)
    }
}
impl<T: Pointee> From<&mut T> for SmartPointer<T>
where
    T: Debug,
{
    fn from(data: &mut T) -> SmartPointer<T> {
        SmartPointer::<T>::from_ref_mut(data)
    }
}
impl<T: Pointee> From<T> for SmartPointer<T>
where
    T: Debug,
{
    fn from(data: T) -> SmartPointer<T> {
        let mut up = SmartPointer::<T>::null();
        up.write(data);
        up
    }
}
impl<T: Pointee> Clone for SmartPointer<T> {
    fn clone(&self) -> SmartPointer<T> {
        SmartPointer::<T>::copy_from_mut_ptr(self.mut_ptr)
    }
}

impl<T: Pointee> Copy for SmartPointer<T> {}

impl<T: Pointee> Pointer for SmartPointer<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:016x}", self.addr())
    }
}

impl<T: Pointee> Debug for SmartPointer<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "SmartPointer{}",
            [
                format!("{:016x}", self.addr()),
                if self.is_not_null() {
                    [format!("[src={:#?}]", self.inner_ref())].join("")
                } else {
                    [format!("[written={}]", self.written)].join("")
                },
            ]
            .join("")
        )
    }
}

impl<T: Pointee + PartialEq> PartialEq<SmartPointer<T>> for SmartPointer<T> {
    fn eq(&self, fles: &SmartPointer<T>) -> bool {
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
impl<T: Pointee + Eq> Eq for SmartPointer<T> {}
impl<T: Pointee + PartialOrd> PartialOrd<SmartPointer<T>> for SmartPointer<T> {
    fn partial_cmp(&self, other: &SmartPointer<T>) -> Option<Ordering> {
        if self.is_null() {
            return None;
        }
        if self.addr() == other.addr() {
            return Some(Ordering::Equal);
        }
        self.inner_ref().partial_cmp(other.inner_ref())
    }
}

impl<T: Pointee + PartialOrd> PartialOrd<T> for SmartPointer<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.is_null() {
            return None;
        }
        self.inner_ref().partial_cmp(other)
    }
}
impl<T: Pointee + PartialEq> PartialEq<T> for SmartPointer<T> {
    fn eq(&self, other: &T) -> bool {
        if self.is_null() {
            return false;
        }
        self.inner_ref().eq(other)
    }
}

impl<T: Pointee + Ord> Ord for SmartPointer<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_null() {
            return Ordering::Less;
        }
        self.inner_ref().cmp(other.inner_ref())
    }
}

impl<T: Pointee + Hash> Hash for SmartPointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner_ref().hash(state)
    }
}
