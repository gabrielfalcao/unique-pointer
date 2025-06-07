use std::iter::Extend;

use crate::{color, Value};
use unique_pointer::{RefCounter, UniquePointer};

/// Rust implementation of lisp's cons cell.
pub struct Cell<'c> {
    head: UniquePointer<Value<'c>>,
    tail: UniquePointer<Cell<'c>>,
    refs: RefCounter,
    length: usize,
}

impl<'c> Cell<'c> {
    pub fn nil() -> Cell<'c> {
        Cell {
            head: UniquePointer::<Value<'c>>::null(),
            tail: UniquePointer::<Cell<'c>>::null(),
            refs: RefCounter::null(),
            length: 0,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.head.is_null() && self.tail.is_null()
    }

    pub fn new(value: Value<'c>) -> Cell<'c> {
        let mut cell = Cell::nil();
        cell.write(value);
        cell
    }

    fn write(&mut self, value: Value<'c>) {
        self.head.write(value);
        self.refs.write(1);
        self.length = 1;
    }

    fn swap_head(&mut self, other: &mut Self) {
        self.head = unsafe {
            let head = other.head.propagate();
            other.head = self.head.propagate();
            head
        };
    }

    fn swap_refs(&mut self, other: &mut Self) {
        self.refs = {
            let refs = other.refs.clone();
            other.refs = self.refs.clone();
            refs
        };
    }

    pub fn head(&self) -> Option<Value<'c>> {
        self.head.try_read()
    }

    pub fn add(&mut self, new: &mut Cell<'c>) {
        new.incr_ref();
        self.incr_ref();
        if self.head.is_null() {
            // when self.head *IS* null:
            unsafe {
                // and `new.head` *IS NOT* null
                if !new.head.is_null() {
                    // swap heads
                    self.swap_head(new);
                }

                // and new.tail *IS NOT* null
                if !new.tail.is_null() {
                    // place `new.tail.head` into `new.head`
                    // <#1>
                    let tail = new.tail.inner_mut();
                    tail.swap_head(new);
                    // </#1>
                    // <#2>
                    // let mut tail = new.tail.read();
                    // let head = tail.head.propagate();
                    // tail.head = UniquePointer::<Value<'c>>::null();
                    // new.head = head;
                    // </#2>
                    self.swap_refs(new);
                    // let refs = new.refs.clone();
                    // new.refs = self.refs.clone();
                    // self.refs = refs;
                }
                self.tail = UniquePointer::read_only(new);
            }
        } else {
            // when self.head *IS NOT* null
            if self.tail.is_null() {
                // dbg!(&self, &new);
                // when self.tail *IS* null
                self.tail = UniquePointer::read_only(new);
            } else {
                // dbg!(self.tail.inner_ref(), &self, &new);
                //  self.tail *IS NOT* null
                self.tail.inner_mut().add(new);
            }
        }
    }

    pub fn pop(&mut self) -> bool {
        if !self.tail.is_null() {
            self.tail.drop_in_place();
            self.tail = UniquePointer::null();
            true
        } else if !self.head.is_null() {
            self.head.drop_in_place();
            self.head = UniquePointer::null();
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() > 0
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        if !self.head.is_null() {
            len += 1
        }
        if let Some(tail) = self.tail() {
            len += tail.len();
        }
        len
    }

    pub fn tail(&self) -> Option<&'c Cell<'c>> {
        self.tail.as_ref()
    }

    pub fn values(&self) -> Vec<Value<'c>> {
        let mut values = Vec::<Value>::new();
        if let Some(head) = self.head() {
            // dbg!(&head);
            values.push(head.clone());
        }
        if let Some(tail) = self.tail() {
            // dbg!(&tail);
            values.extend(tail.values());
        }
        values
    }

    fn incr_ref(&mut self) {
        self.refs += 1;
        if !self.tail.is_null() {
            if let Some(tail) = self.tail.as_mut() {
                tail.incr_ref();
            }
        }
    }

    fn decr_ref(&mut self) {
        self.refs -= 1;
        if !self.tail.is_null() {
            if let Some(tail) = self.tail.as_mut() {
                tail.decr_ref();
            }
        }
    }

    fn dealloc(&mut self) {
        if self.refs > 0 {
            self.decr_ref();
        } else {
            self.head.drop_in_place();
            self.tail.drop_in_place();
        }
    }
}

impl<'c> From<Value<'c>> for Cell<'c> {
    fn from(value: Value<'c>) -> Cell<'c> {
        Cell::new(value)
    }
}
impl<'c> From<&'c str> for Cell<'c> {
    fn from(value: &'c str) -> Cell<'c> {
        let value = Value::from(value);
        Cell::new(value)
    }
}
impl<'c> From<u8> for Cell<'c> {
    fn from(value: u8) -> Cell<'c> {
        Cell::new(Value::Byte(value))
    }
}
impl<'c> From<u64> for Cell<'c> {
    fn from(value: u64) -> Cell<'c> {
        if value < u8::MAX.into() {
            Cell::new(Value::Byte(value as u8))
        } else {
            Cell::new(Value::UInt(value))
        }
    }
}
impl<'c> From<i32> for Cell<'c> {
    fn from(value: i32) -> Cell<'c> {
        if let Ok(value) = TryInto::<u64>::try_into(value) {
            Cell::new(Value::UInt(value))
        } else {
            Cell::new(Value::Int(value.into()))
        }
    }
}
impl<'c> From<i64> for Cell<'c> {
    fn from(value: i64) -> Cell<'c> {
        Cell::new(Value::from(value))
    }
}

impl<'c> PartialEq<Cell<'c>> for Cell<'c> {
    fn eq(&self, other: &Cell<'c>) -> bool {
        if self.head.is_null() == other.head.is_null() {
            true
        } else if let Some(head) = self.head() {
            if let Some(value) = other.head() {
                return head == value && (self.tail() == other.tail());
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<'c> Default for Cell<'c> {
    fn default() -> Cell<'c> {
        Cell::nil()
    }
}
impl<'c> Clone for Cell<'c> {
    fn clone(&self) -> Cell<'c> {
        let mut cell = Cell::nil();
        cell.refs = self.refs.clone();
        if self.head.is_not_null() {
            cell.head = self.head.clone();
        }
        if self.tail.is_not_null() {
            cell.tail = self.tail.clone();
        }
        cell
    }
}
impl<'c> Drop for Cell<'c> {
    fn drop(&mut self) {
        self.dealloc();
    }
}

impl std::fmt::Debug for Cell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            [
                crate::color::reset(""),
                crate::color::fore("Cell", 87),
                crate::color::fore("@", 231),
                crate::color::ref_addr(self),
                crate::color::ansi(format!("[refs={}]", self.refs), 220, 16),
                format!(
                    "[{}]",
                    if self.is_nil() {
                        format!("head and tail={}", color::fore("null", 196))
                    } else {
                        [
                            if self.head.is_null() {
                                format!("head: {}", color::fore("null", 196))
                            } else {
                                crate::color::ansi(format!("head={:#?}", self.head), 231, 16)
                            },
                            if self.tail.is_null() {
                                format!("tail: {}", color::fore("null", 196))
                            } else {
                                crate::color::fore(format!("tail={:#?}", self.tail), 82)
                            },
                        ]
                        .join(" | ")
                    }
                )
            ]
            .join("")
        )
    }
}
