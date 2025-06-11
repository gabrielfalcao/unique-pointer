#![allow(unused)]
#![feature(intra_doc_pointers)]
#![doc(issue_tracker_base_url = "https://github.com/gabrielfalcao/unique-pointer/issues/")]
//! [UniquePointer] is an experimental data structure that makes
//! extensive use of unsafe rust to provide a shared pointer
//! throughout the runtime of a rust program as transparently as
//! possible.
//!
//! # Binary Tree Example
//!
//! ### Binary Tree Implementation
//!
//! ```
//! use unique_pointer::{RefCounter, UniquePointer};
//!
//! use std::borrow::Cow;
//! use std::convert::{AsMut, AsRef};
//! use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
//!
//! #  #[derive(Clone, PartialOrd, Ord, Default, PartialEq, Eq)]
//! #  pub enum Value<'c> {
//! #      #[default]
//! #      Nil,
//! #      String(Cow<'c, str>),
//! #      Byte(u8),
//! #      UInt(u64),
//! #      Int(i64),
//! #  }
//! #  impl<'c> Value<'_> {
//! #      pub fn nil() -> Value<'c> {
//! #          Value::Nil
//! #      }
//! #
//! #      pub fn is_nil(&self) -> bool {
//! #          if *self == Value::Nil {
//! #              true
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> Drop for Value<'c> {
//! #      fn drop(&mut self) {}
//! #  }
//! #
//! #  impl std::fmt::Display for Value<'_> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              match self {
//! #                  Value::Nil => "nil".to_string(),
//! #                  Value::String(h) => format!("{}", h),
//! #                  Value::Byte(h) => format!("{}", h),
//! #                  Value::UInt(h) => format!("{}", h),
//! #                  Value::Int(h) => format!("{}", h),
//! #              }
//! #          )
//! #      }
//! #  }
//! #  impl std::fmt::Debug for Value<'_> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              match self {
//! #                  Value::Nil => "nil".to_string(),
//! #                  Value::String(h) => format!("{:#?}", h),
//! #                  Value::Byte(h) => format!("{}u8", h),
//! #                  Value::UInt(h) => format!("{}u64", h),
//! #                  Value::Int(h) => format!("{}i64", h),
//! #              }
//! #          )
//! #      }
//! #  }
//! #
//! #  impl<'c> From<u8> for Value<'c> {
//! #      fn from(value: u8) -> Value<'c> {
//! #          Value::Byte(value)
//! #      }
//! #  }
//! #  impl<'c> From<u64> for Value<'c> {
//! #      fn from(value: u64) -> Value<'c> {
//! #          Value::UInt(value)
//! #      }
//! #  }
//! #  impl<'c> From<i64> for Value<'c> {
//! #      fn from(value: i64) -> Value<'c> {
//! #          Value::Int(value)
//! #      }
//! #  }
//! #  impl<'c> From<&'c str> for Value<'c> {
//! #      fn from(value: &'c str) -> Value<'c> {
//! #          Value::String(Cow::from(value))
//! #      }
//! #  }
//! #  impl<'c> From<Cow<'c, str>> for Value<'c> {
//! #      fn from(value: Cow<'c, str>) -> Value<'c> {
//! #          Value::from(value.into_owned())
//! #      }
//! #  }
//! #  impl<'c> From<&'c mut str> for Value<'c> {
//! #      fn from(value: &'c mut str) -> Value<'c> {
//! #          Value::String(Cow::<'c, str>::Borrowed(&*value))
//! #      }
//! #  }
//! #  impl<'c> From<String> for Value<'c> {
//! #      fn from(value: String) -> Value<'c> {
//! #          Value::String(Cow::from(value))
//! #      }
//! #  }
//! #  impl<'c> From<Option<String>> for Value<'c> {
//! #      fn from(value: Option<String>) -> Value<'c> {
//! #          match value {
//! #              None => Value::Nil,
//! #              Some(value) => Value::from(value),
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> AsRef<Value<'c>> for Value<'c> {
//! #      fn as_ref(&self) -> &Value<'c> {
//! #          unsafe { &*self }
//! #      }
//! #  }
//! #  impl<'c> AsMut<Value<'c>> for Value<'c> {
//! #      fn as_mut(&mut self) -> &mut Value<'c> {
//! #          unsafe { &mut *self }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&Value<'c>> for Value<'c> {
//! #      fn eq(&self, other: &&Value<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          self == other
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&mut Value<'c>> for Value<'c> {
//! #      fn eq(&self, other: &&mut Value<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          self == other
//! #      }
//! #  }
//! #
//! pub struct Node<'c> {
//!     pub parent: UniquePointer<Node<'c>>,
//!     pub left: UniquePointer<Node<'c>>,
//!     pub right: UniquePointer<Node<'c>>,
//!     pub item: UniquePointer<Value<'c>>,
//!     refs: RefCounter,
//! }
//!
//! impl<'c> Node<'c> {
//!     pub fn nil() -> Node<'c> {
//!         Node {
//!             parent: UniquePointer::<Node<'c>>::null(),
//!             left: UniquePointer::<Node<'c>>::null(),
//!             right: UniquePointer::<Node<'c>>::null(),
//!             item: UniquePointer::<Value<'c>>::null(),
//!             refs: RefCounter::new(),
//!         }
//!     }
//!
//!     pub fn is_nil(&self) -> bool {
//!         self.item.is_null()
//!             && self.left.is_null()
//!             && self.right.is_null()
//!             && self.parent.is_null()
//!             && self.refs <= 1
//!     }
//!
//!     pub fn new(value: Value<'c>) -> Node<'c> {
//!         let mut node = Node::nil();
//!         unsafe {
//!             node.item.write(value);
//!         }
//!         node
//!     }
//!
//!     pub fn parent(&self) -> Option<&'c Node<'c>> {
//!         self.parent.as_ref()
//!     }
//!
//!     pub fn parent_mut(&mut self) -> Option<&'c mut Node<'c>> {
//!         self.parent.as_mut()
//!     }
//!
//!     pub fn item(&self) -> Value<'c> {
//!         self.value().unwrap_or_default()
//!     }
//!
//!     pub fn id(&self) -> String {
//!         format!(
//!             "{}{}",
//!             if self.item.is_null() {
//!                 format!("Null Node {:p}", self)
//!             } else {
//!                 format!("Node {}", self.item())
//!             },
//!             format!(" ({} referefences)", self.refs)
//!         )
//!     }
//!
//!     pub fn value(&self) -> Option<Value<'c>> {
//!         if self.item.is_null() {
//!             None
//!         } else {
//!             unsafe {
//!                 if let Some(value) = self.item.as_ref() {
//!                     Some(value.clone())
//!                 } else {
//!                     None
//!                 }
//!             }
//!         }
//!     }
//!
//!     pub fn parent_value(&self) -> Option<Value<'c>> {
//!         if let Some(parent) = self.parent() {
//!             parent.value()
//!         } else {
//!             None
//!         }
//!     }
//!
//!     pub fn set_left(&mut self, left: &mut Node<'c>) {
//!         self.incr_ref();
//!         left.parent = self.ptr();
//!         self.left = left.ptr();
//!         left.incr_ref();
//!     }
//!
//!     pub fn set_right(&mut self, right: &mut Node<'c>) {
//!         self.incr_ref();
//!         right.parent = self.ptr();
//!         self.right = right.ptr();
//!         right.incr_ref();
//!     }
//!
//!     pub fn delete_left(&mut self) {
//!         if self.left.is_null() {
//!             return;
//!         }
//!         let left = self.left.inner_mut();
//!         left.decr_ref();
//!         self.left.dealloc(true);
//!         self.left = UniquePointer::null();
//!     }
//!
//!     pub fn left(&self) -> Option<&'c Node<'c>> {
//!         let left = self.left.as_ref();
//!         left
//!     }
//!
//!     pub fn left_mut(&mut self) -> Option<&'c mut Node<'c>> {
//!         self.left.as_mut()
//!     }
//!
//!     pub fn left_value(&self) -> Option<Value<'c>> {
//!         if let Some(left) = self.left() {
//!             left.value()
//!         } else {
//!             None
//!         }
//!     }
//!
//!     pub fn delete_right(&mut self) {
//!         if self.right.is_null() {
//!             return;
//!         }
//!         let right = self.right.inner_mut();
//!         right.decr_ref();
//!         self.right.dealloc(true);
//!         self.right = UniquePointer::null();
//!     }
//!
//!     pub fn right(&self) -> Option<&'c Node<'c>> {
//!         self.right.as_ref()
//!     }
//!
//!     pub fn right_mut(&mut self) -> Option<&'c mut Node<'c>> {
//!         self.right.as_mut()
//!     }
//!
//!     pub fn right_value(&self) -> Option<Value<'c>> {
//!         if let Some(right) = self.right() {
//!             right.value()
//!         } else {
//!             None
//!         }
//!     }
//!
//!     pub fn height(&self) -> usize {
//!         let mut node = self;
//!         let mut vertices = 0;
//!
//!         while !node.left.is_null() {
//!             node = node.left.inner_ref();
//!             vertices += 1;
//!         }
//!         vertices
//!     }
//!
//!     pub fn depth(&self) -> usize {
//!         let mut node = self;
//!         if self.parent.is_null() {
//!             return 0;
//!         }
//!         let mut vertices = 0;
//!
//!         while !node.parent.is_null() {
//!             node = node.parent.inner_ref();
//!             vertices += 1;
//!         }
//!         vertices
//!     }
//!
//!     pub fn leaf(&self) -> bool {
//!         self.left.is_null() && self.right.is_null()
//!     }
//!
//!     pub fn addr(&self) -> usize {
//!         (self as *const Node<'c>).addr()
//!     }
//!
//!     pub fn left_addr(&self) -> usize {
//!         self.left.addr()
//!     }
//!
//!     pub fn right_addr(&self) -> usize {
//!         self.right.addr()
//!     }
//!
//!     pub fn parent_addr(&self) -> usize {
//!         self.parent.addr()
//!     }
//!
//!     pub fn refs(&self) -> usize {
//!         *self.refs
//!     }
//!
//!     pub fn subtree_first(&self) -> &'c Node<'c> {
//!         if self.left.is_null() {
//!             let node = self as *const Node<'c>;
//!             return unsafe { node.as_ref().unwrap() };
//!         }
//!
//!         let mut subtree_first = self.left.cast_mut();
//!
//!         loop {
//!             unsafe {
//!                 let node = &*subtree_first;
//!                 if node.left.is_null() {
//!                     break;
//!                 }
//!                 subtree_first = node.left.cast_mut()
//!             }
//!         }
//!         unsafe { subtree_first.as_mut().unwrap() }
//!     }
//!
//!     pub fn successor(&self) -> &'c Node<'c> {
//!         if !self.right.is_null() {
//!             return unsafe { self.right.as_ref().unwrap() }.subtree_first();
//!         }
//!
//!         if let Some(parent) = self.parent() {
//!             if parent.parent.is_null() {
//!                 return self.subtree_first();
//!             }
//!         }
//!         let mut successor = self as *const Node<'c>;
//!         let mut node = unsafe { &*successor };
//!         loop {
//!             if node.left() == Some(self) {
//!                 break;
//!             }
//!             if !node.parent.is_null() {
//!                 successor = node.parent.cast_const();
//!                 node = unsafe { &*successor };
//!             } else {
//!                 break;
//!             };
//!         }
//!         unsafe { &*successor }
//!     }
//!
//!     pub fn subtree_first_mut(&mut self) -> &'c mut Node<'c> {
//!         if self.left.is_null() {
//!             let node = self as *mut Node<'c>;
//!             return {
//!                 let node = unsafe {
//!                     let node = &mut *node;
//!                     node
//!                 };
//!                 unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//!             };
//!         }
//!
//!         let mut subtree_first = &mut self.left;
//!
//!         loop {
//!             unsafe {
//!                 let node = subtree_first.inner_mut();
//!                 if node.left.is_null() {
//!                     break;
//!                 }
//!                 subtree_first = &mut node.left;
//!             }
//!         }
//!
//!         subtree_first.inner_mut()
//!     }
//!
//!     pub fn successor_mut(&mut self) -> &'c mut Node<'c> {
//!         if !self.right.is_null() {
//!             return self.right.inner_mut().subtree_first_mut();
//!         }
//!
//!         if let Some(parent) = self.parent() {
//!             if parent.parent.is_null() {
//!                 return self.subtree_first_mut();
//!             }
//!         }
//!         let mut successor = self as *mut Node<'c>;
//!         let mut node = {
//!             let node = unsafe {
//!                 let node = &mut *successor;
//!                 node
//!             };
//!             unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//!         };
//!
//!         loop {
//!             if node.left() == Some(self) {
//!                 break;
//!             }
//!             if !node.parent.is_null() {
//!                 successor = node.parent.cast_mut();
//!                 node = {
//!                     let node = unsafe {
//!                         let node = &mut *successor;
//!                         node
//!                     };
//!                     unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//!                 };
//!             } else {
//!                 break;
//!             };
//!         }
//!         {
//!             let node = unsafe {
//!                 let node = &mut *successor;
//!                 node
//!             };
//!             unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//!         }
//!     }
//!
//!     pub fn subtree_insert_after(&mut self, new: &mut Node<'c>) {
//!         if self.right.is_null() {
//!             self.set_right(new);
//!         } else {
//!             let successor = self.successor_mut();
//!             successor.set_left(new);
//!         }
//!     }
//!
//!     pub fn predecessor(&self) -> &'c Node<'c> {
//!         let mut predecessor = self as *const Node<'c>;
//!         let mut node = {
//!             let node = unsafe {
//!                 let node = &*predecessor;
//!                 node
//!             };
//!             unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//!         };
//!
//!         loop {
//!             if !node.left.is_null() {
//!                 predecessor = node.left.cast_const();
//!                 node = {
//!                     let node = unsafe {
//!                         let node = &*predecessor;
//!                         node
//!                     };
//!                     unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//!                 };
//!                 if !node.right.is_null() {
//!                     predecessor = node.right.cast_const();
//!                     node = {
//!                         let node = unsafe {
//!                             let node = &*predecessor;
//!                             node
//!                         };
//!                         unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//!                     };
//!                 }
//!                 break;
//!             } else if !node.parent.is_null() {
//!                 predecessor = node.parent.cast_const();
//!                 node = {
//!                     let node = unsafe {
//!                         let node = &*predecessor;
//!                         node
//!                     };
//!                     unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//!                 };
//!                 if let Some(right) = node.right() {
//!                     if right == self {
//!                         break;
//!                     }
//!                 }
//!             }
//!         }
//!         node = {
//!             let node = unsafe {
//!                 let node = &*predecessor;
//!                 node
//!             };
//!             unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//!         };
//!         node
//!     }
//!
//!     pub fn predecessor_mut(&mut self) -> &'c mut Node<'c> {
//!         let mut predecessor = UniquePointer::<Node<'c>>::from_ref_mut(self);
//!         let mut node = predecessor.inner_mut();
//!
//!         loop {
//!             if !node.left.is_null() {
//!                 predecessor = node.left.clone();
//!                 node = predecessor.inner_mut();
//!                 if !node.right.is_null() {
//!                     predecessor = node.right.clone();
//!                     node = predecessor.inner_mut();
//!                 }
//!                 break;
//!             } else if !node.parent.is_null() {
//!                 predecessor = node.parent.clone();
//!                 node = predecessor.inner_mut();
//!
//!                 if let Some(right) = node.right() {
//!                     if right == self {
//!                         break;
//!                     }
//!                 }
//!             }
//!         }
//!         predecessor.inner_mut()
//!     }
//!
//!     pub fn dealloc(&mut self) {
//!         if self.refs > 0 {
//!             self.decr_ref();
//!         } else {
//!             if !self.parent.is_null() {
//!                 self.parent.drop_in_place();
//!                 // self.parent = UniquePointer::null();
//!             }
//!             if !self.left.is_null() {
//!                 self.left.drop_in_place();
//!                 // self.left = UniquePointer::null();
//!             }
//!             if !self.right.is_null() {
//!                 self.right.drop_in_place();
//!                 // self.right = UniquePointer::null();
//!             }
//!             if !self.item.is_null() {
//!                 self.item.drop_in_place();
//!                 // self.item = UniquePointer::null();
//!             }
//!         }
//!     }
//!
//!     pub fn swap_item(&mut self, other: &mut Self) {
//!         unsafe {
//!             self.item.swap(&mut other.item);
//!         };
//!     }
//! }
//!
//! pub fn subtree_delete<'c>(node: &mut Node<'c>) {
//!     if node.leaf() {
//!         node.decr_ref();
//!         if node.parent.is_not_null() {
//!             unsafe {
//!                 let parent = node.parent.inner_mut();
//!                 let delete_left = if let Some(parents_left_child) = parent.left() {
//!                     parents_left_child == node
//!                 } else {
//!                     false
//!                 };
//!                 if delete_left {
//!                     parent.left.dealloc(true);
//!                     parent.left = UniquePointer::null();
//!                 } else {
//!                     parent.right.dealloc(true);
//!                     parent.right = UniquePointer::null();
//!                 }
//!             }
//!             node.parent.dealloc(true);
//!             node.parent = UniquePointer::null();
//!         }
//!         node.refs.reset();
//!         node.parent = UniquePointer::<Node<'c>>::null();
//!         return;
//!     } else {
//!         let predecessor = node.predecessor_mut();
//!         predecessor.swap_item(node);
//!         subtree_delete(predecessor);
//!     }
//! }
//!
//! // Node private methods
//! impl<'c> Node<'c> {
//!     pub fn ptr(&self) -> UniquePointer<Node<'c>> {
//!         let ptr = UniquePointer::copy_from_ref(self, *self.refs);
//!         ptr
//!     }
//!
//!     fn incr_ref(&mut self) {
//!         self.refs += 1;
//!         let mut node = self;
//!         while !node.parent.is_null() {
//!             unsafe {
//!                 node = node.parent.inner_mut();
//!                 node.refs += 1;
//!             }
//!         }
//!     }
//!
//!     fn decr_ref(&mut self) {
//!         self.refs -= 1;
//!         let mut node = self;
//!         while !node.parent.is_null() {
//!             unsafe {
//!                 node = node.parent.inner_mut();
//!                 node.refs -= 1;
//!             }
//!         }
//!     }
//!
//!     fn item_eq(&self, other: &Node<'c>) -> bool {
//!         if self.item.addr() == other.item.addr() {
//!             self.item.addr() == other.item.addr()
//!         } else {
//!             self.value() == other.value()
//!         }
//!     }
//! }
//!
//! impl<'c> PartialEq<Node<'c>> for Node<'c> {
//!     fn eq(&self, other: &Node<'c>) -> bool {
//!         if self.item_eq(other) {
//!             let eq = self.value().unwrap_or_default() == other.value().unwrap_or_default();
//!             eq
//!         } else {
//!             false
//!         }
//!     }
//! }
//!
//! impl<'c> PartialEq<&mut Node<'c>> for Node<'c> {
//!     fn eq(&self, other: &&mut Node<'c>) -> bool {
//!         let other = unsafe { &**other };
//!         if self.item_eq(other) {
//!             let eq = self.value().unwrap_or_default() == other.value().unwrap_or_default();
//!             eq
//!         } else {
//!             false
//!         }
//!     }
//! }
//!
//! impl<'c> Drop for Node<'c> {
//!     fn drop(&mut self) {
//!         self.dealloc();
//!     }
//! }
//!
//! impl<'c> Clone for Node<'c> {
//!     fn clone(&self) -> Node<'c> {
//!         let mut node = Node::nil();
//!         node.refs = self.refs.clone();
//!         if self.parent.is_not_null() {
//!             node.parent = self.parent.clone();
//!         }
//!         if self.left.is_not_null() {
//!             node.left = self.left.clone();
//!         }
//!         if self.right.is_not_null() {
//!             node.right = self.right.clone();
//!         }
//!         if !self.item.is_null() {
//!             node.item = self.item.clone();
//!         }
//!         node
//!     }
//! }
//!
//! impl<'c> AsRef<Node<'c>> for Node<'c> {
//!     fn as_ref(&self) -> &'c Node<'c> {
//!         unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(self) }
//!     }
//! }
//! impl<'c> AsMut<Node<'c>> for Node<'c> {
//!     fn as_mut(&mut self) -> &'c mut Node<'c> {
//!         self.incr_ref();
//!         let node = unsafe {
//!             let node = &mut *self as *mut Node<'c>;
//!             node
//!         };
//!         unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(self) }
//!     }
//! }
//! impl<'c> std::fmt::Display for Node<'c> {
//!     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//!         write!(f, "{}", self.id())
//!     }
//! }
//! impl<'c> std::fmt::Debug for Node<'c> {
//!     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//!         write!(
//!             f,
//!             "{}",
//!             [
//!                 format!("Node@"),
//!                 format!("{:016x}", self.addr()),
//!                 format!("[refs={}]", *self.refs),
//!                 if self.item.is_null() {
//!                     format!("null")
//!                 } else {
//!                     format!(
//!                         "[item={}]",
//!                         self.value()
//!                             .map(|value| format!("{:#?}", value))
//!                             .unwrap_or_else(|| format!("empty"))
//!                     )
//!                 },
//!                 if self.parent.is_null() {
//!                     String::new()
//!                 } else {
//!                     format!(
//!                         "(parent:{})",
//!                         if self.parent.is_null() {
//!                             format!("null")
//!                         } else {
//!                             self.parent_value()
//!                                 .map(|parent_value| format!("{:#?}", parent_value))
//!                                 .unwrap_or_else(|| format!("empty"))
//!                         }
//!                     )
//!                 },
//!                 if self.left.is_null() && self.right.is_null() {
//!                     String::new()
//!                 } else {
//!                     format!(
//!                         "[left:{} | right:{}]",
//!                         if self.left.is_null() {
//!                             format!("null")
//!                         } else {
//!                             self.left_value()
//!                                 .map(|left_value| format!("{:#?}", left_value))
//!                                 .unwrap_or_else(|| format!("empty"))
//!                         },
//!                         if self.right.is_null() {
//!                             format!("null")
//!                         } else {
//!                             self.right_value()
//!                                 .map(|right_value| format!("{:#?}", right_value))
//!                                 .unwrap_or_else(|| format!("empty"))
//!                         }
//!                     )
//!                 }
//!             ]
//!             .join("")
//!         )
//!     }
//! }
//! ```
//!
//! ### Testing the Binary Tree
//!
//! ```
//! #  use std::borrow::Cow;
//! #  use std::convert::{AsMut, AsRef};
//! #  use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
//! #  use unique_pointer::{UniquePointer, RefCounter};
//! #  #[derive(Clone, PartialOrd, Ord, Default, PartialEq, Eq)]
//! #  pub enum Value<'c> {
//! #      #[default]
//! #      Nil,
//! #      String(Cow<'c, str>),
//! #      Byte(u8),
//! #      UInt(u64),
//! #      Int(i64),
//! #  }
//! #  impl<'c> Value<'_> {
//! #      pub fn nil() -> Value<'c> {
//! #          Value::Nil
//! #      }
//! #
//! #      pub fn is_nil(&self) -> bool {
//! #          if *self == Value::Nil {
//! #              true
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> Drop for Value<'c> {
//! #      fn drop(&mut self) {}
//! #  }
//! #
//! #  impl std::fmt::Display for Value<'_> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              match self {
//! #                  Value::Nil => "nil".to_string(),
//! #                  Value::String(h) => format!("{}", h),
//! #                  Value::Byte(h) => format!("{}", h),
//! #                  Value::UInt(h) => format!("{}", h),
//! #                  Value::Int(h) => format!("{}", h),
//! #              }
//! #          )
//! #      }
//! #  }
//! #  impl std::fmt::Debug for Value<'_> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              match self {
//! #                  Value::Nil => "nil".to_string(),
//! #                  Value::String(h) => format!("{:#?}", h),
//! #                  Value::Byte(h) => format!("{}u8", h),
//! #                  Value::UInt(h) => format!("{}u64", h),
//! #                  Value::Int(h) => format!("{}i64", h),
//! #              }
//! #          )
//! #      }
//! #  }
//! #
//! #  impl<'c> From<u8> for Value<'c> {
//! #      fn from(value: u8) -> Value<'c> {
//! #          Value::Byte(value)
//! #      }
//! #  }
//! #  impl<'c> From<u64> for Value<'c> {
//! #      fn from(value: u64) -> Value<'c> {
//! #          Value::UInt(value)
//! #      }
//! #  }
//! #  impl<'c> From<i64> for Value<'c> {
//! #      fn from(value: i64) -> Value<'c> {
//! #          Value::Int(value)
//! #      }
//! #  }
//! #  impl<'c> From<&'c str> for Value<'c> {
//! #      fn from(value: &'c str) -> Value<'c> {
//! #          Value::String(Cow::from(value))
//! #      }
//! #  }
//! #  impl<'c> From<Cow<'c, str>> for Value<'c> {
//! #      fn from(value: Cow<'c, str>) -> Value<'c> {
//! #          Value::from(value.into_owned())
//! #      }
//! #  }
//! #  impl<'c> From<&'c mut str> for Value<'c> {
//! #      fn from(value: &'c mut str) -> Value<'c> {
//! #          Value::String(Cow::<'c, str>::Borrowed(&*value))
//! #      }
//! #  }
//! #  impl<'c> From<String> for Value<'c> {
//! #      fn from(value: String) -> Value<'c> {
//! #          Value::String(Cow::from(value))
//! #      }
//! #  }
//! #  impl<'c> From<Option<String>> for Value<'c> {
//! #      fn from(value: Option<String>) -> Value<'c> {
//! #          match value {
//! #              None => Value::Nil,
//! #              Some(value) => Value::from(value),
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> AsRef<Value<'c>> for Value<'c> {
//! #      fn as_ref(&self) -> &Value<'c> {
//! #          unsafe { &*self }
//! #      }
//! #  }
//! #  impl<'c> AsMut<Value<'c>> for Value<'c> {
//! #      fn as_mut(&mut self) -> &mut Value<'c> {
//! #          unsafe { &mut *self }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&Value<'c>> for Value<'c> {
//! #      fn eq(&self, other: &&Value<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          self == other
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&mut Value<'c>> for Value<'c> {
//! #      fn eq(&self, other: &&mut Value<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          self == other
//! #      }
//! #  }
//! #
//! #
//! #  pub struct Node<'c> {
//! #      pub parent: UniquePointer<Node<'c>>,
//! #      pub left: UniquePointer<Node<'c>>,
//! #      pub right: UniquePointer<Node<'c>>,
//! #      pub item: UniquePointer<Value<'c>>,
//! #      refs: RefCounter,
//! #  }
//! #
//! #  impl<'c> Node<'c> {
//! #      pub fn nil() -> Node<'c> {
//! #          Node {
//! #              parent: UniquePointer::<Node<'c>>::null(),
//! #              left: UniquePointer::<Node<'c>>::null(),
//! #              right: UniquePointer::<Node<'c>>::null(),
//! #              item: UniquePointer::<Value<'c>>::null(),
//! #              refs: RefCounter::new(),
//! #          }
//! #      }
//! #
//! #      pub fn is_nil(&self) -> bool {
//! #          self.item.is_null()
//! #              && self.left.is_null()
//! #              && self.right.is_null()
//! #              && self.parent.is_null()
//! #              && self.refs <= 1
//! #      }
//! #
//! #      pub fn new(value: Value<'c>) -> Node<'c> {
//! #          let mut node = Node::nil();
//! #          unsafe {
//! #              node.item.write(value);
//! #          }
//! #          node
//! #      }
//! #
//! #      pub fn parent(&self) -> Option<&'c Node<'c>> {
//! #          self.parent.as_ref()
//! #      }
//! #
//! #      pub fn parent_mut(&mut self) -> Option<&'c mut Node<'c>> {
//! #          self.parent.as_mut()
//! #      }
//! #
//! #      pub fn item(&self) -> Value<'c> {
//! #          self.value().unwrap_or_default()
//! #      }
//! #
//! #      pub fn id(&self) -> String {
//! #          format!(
//! #              "{}{}",
//! #              if self.item.is_null() {
//! #                  format!("Null Node {:p}", self)
//! #              } else {
//! #                  format!("Node {}", self.item())
//! #              },
//! #              format!(" ({} referefences)", self.refs)
//! #          )
//! #      }
//! #
//! #      pub fn value(&self) -> Option<Value<'c>> {
//! #          if self.item.is_null() {
//! #              None
//! #          } else {
//! #              unsafe {
//! #                  if let Some(value) = self.item.as_ref() {
//! #                      Some(value.clone())
//! #                  } else {
//! #                      None
//! #                  }
//! #              }
//! #          }
//! #      }
//! #
//! #      pub fn parent_value(&self) -> Option<Value<'c>> {
//! #          if let Some(parent) = self.parent() {
//! #              parent.value()
//! #          } else {
//! #              None
//! #          }
//! #      }
//! #
//! #      pub fn set_left(&mut self, left: &mut Node<'c>) {
//! #          self.incr_ref();
//! #          left.parent = self.ptr();
//! #          self.left = left.ptr();
//! #          left.incr_ref();
//! #      }
//! #
//! #      pub fn set_right(&mut self, right: &mut Node<'c>) {
//! #          self.incr_ref();
//! #          right.parent = self.ptr();
//! #          self.right = right.ptr();
//! #          right.incr_ref();
//! #      }
//! #
//! #      pub fn delete_left(&mut self) {
//! #          if self.left.is_null() {
//! #              return;
//! #          }
//! #          let left = self.left.inner_mut();
//! #          left.decr_ref();
//! #          self.left.dealloc(true);
//! #          self.left = UniquePointer::null();
//! #      }
//! #
//! #      pub fn left(&self) -> Option<&'c Node<'c>> {
//! #          let left = self.left.as_ref();
//! #          left
//! #      }
//! #
//! #      pub fn left_mut(&mut self) -> Option<&'c mut Node<'c>> {
//! #          self.left.as_mut()
//! #      }
//! #
//! #      pub fn left_value(&self) -> Option<Value<'c>> {
//! #          if let Some(left) = self.left() {
//! #              left.value()
//! #          } else {
//! #              None
//! #          }
//! #      }
//! #
//! #      pub fn delete_right(&mut self) {
//! #          if self.right.is_null() {
//! #              return;
//! #          }
//! #          let right = self.right.inner_mut();
//! #          right.decr_ref();
//! #          self.right.dealloc(true);
//! #          self.right = UniquePointer::null();
//! #      }
//! #
//! #      pub fn right(&self) -> Option<&'c Node<'c>> {
//! #          self.right.as_ref()
//! #      }
//! #
//! #      pub fn right_mut(&mut self) -> Option<&'c mut Node<'c>> {
//! #          self.right.as_mut()
//! #      }
//! #
//! #      pub fn right_value(&self) -> Option<Value<'c>> {
//! #          if let Some(right) = self.right() {
//! #              right.value()
//! #          } else {
//! #              None
//! #          }
//! #      }
//! #
//! #      pub fn height(&self) -> usize {
//! #          let mut node = self;
//! #          let mut vertices = 0;
//! #
//! #          while !node.left.is_null() {
//! #              node = node.left.inner_ref();
//! #              vertices += 1;
//! #          }
//! #          vertices
//! #      }
//! #
//! #      pub fn depth(&self) -> usize {
//! #          let mut node = self;
//! #          if self.parent.is_null() {
//! #              return 0;
//! #          }
//! #          let mut vertices = 0;
//! #
//! #          while !node.parent.is_null() {
//! #              node = node.parent.inner_ref();
//! #              vertices += 1;
//! #          }
//! #          vertices
//! #      }
//! #
//! #      pub fn leaf(&self) -> bool {
//! #          self.left.is_null() && self.right.is_null()
//! #      }
//! #
//! #      pub fn addr(&self) -> usize {
//! #          (self as *const Node<'c>).addr()
//! #      }
//! #
//! #      pub fn left_addr(&self) -> usize {
//! #          self.left.addr()
//! #      }
//! #
//! #      pub fn right_addr(&self) -> usize {
//! #          self.right.addr()
//! #      }
//! #
//! #      pub fn parent_addr(&self) -> usize {
//! #          self.parent.addr()
//! #      }
//! #
//! #      pub fn refs(&self) -> usize {
//! #          *self.refs
//! #      }
//! #
//! #      pub fn subtree_first(&self) -> &'c Node<'c> {
//! #          if self.left.is_null() {
//! #              let node = self as *const Node<'c>;
//! #              return unsafe { node.as_ref().unwrap() };
//! #          }
//! #
//! #          let mut subtree_first = self.left.cast_mut();
//! #
//! #          loop {
//! #              unsafe {
//! #                  let node = &*subtree_first;
//! #                  if node.left.is_null() {
//! #                      break;
//! #                  }
//! #                  subtree_first = node.left.cast_mut()
//! #              }
//! #          }
//! #          unsafe { subtree_first.as_mut().unwrap() }
//! #      }
//! #
//! #      pub fn successor(&self) -> &'c Node<'c> {
//! #          if !self.right.is_null() {
//! #              return unsafe { self.right.as_ref().unwrap() }.subtree_first();
//! #          }
//! #
//! #          if let Some(parent) = self.parent() {
//! #              if parent.parent.is_null() {
//! #                  return self.subtree_first();
//! #              }
//! #          }
//! #          let mut successor = self as *const Node<'c>;
//! #          let mut node = unsafe { &*successor };
//! #          loop {
//! #              if node.left() == Some(self) {
//! #                  break;
//! #              }
//! #              if !node.parent.is_null() {
//! #                  successor = node.parent.cast_const();
//! #                  node = unsafe { &*successor };
//! #              } else {
//! #                  break;
//! #              };
//! #          }
//! #          unsafe { &*successor }
//! #      }
//! #
//! #      pub fn subtree_first_mut(&mut self) -> &'c mut Node<'c> {
//! #          if self.left.is_null() {
//! #              let node = self as *mut Node<'c>;
//! #              return {
//! #                  let node = unsafe {
//! #                      let node = &mut *node;
//! #                      node
//! #                  };
//! #                  unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #              };
//! #          }
//! #
//! #          let mut subtree_first = &mut self.left;
//! #
//! #          loop {
//! #              unsafe {
//! #                  let node = subtree_first.inner_mut();
//! #                  if node.left.is_null() {
//! #                      break;
//! #                  }
//! #                  subtree_first = &mut node.left;
//! #              }
//! #          }
//! #
//! #          subtree_first.inner_mut()
//! #      }
//! #
//! #      pub fn successor_mut(&mut self) -> &'c mut Node<'c> {
//! #          if !self.right.is_null() {
//! #              return self.right.inner_mut().subtree_first_mut();
//! #          }
//! #
//! #          if let Some(parent) = self.parent() {
//! #              if parent.parent.is_null() {
//! #                  return self.subtree_first_mut();
//! #              }
//! #          }
//! #          let mut successor = self as *mut Node<'c>;
//! #          let mut node = {
//! #              let node = unsafe {
//! #                  let node = &mut *successor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #          };
//! #
//! #          loop {
//! #              if node.left() == Some(self) {
//! #                  break;
//! #              }
//! #              if !node.parent.is_null() {
//! #                  successor = node.parent.cast_mut();
//! #                  node = {
//! #                      let node = unsafe {
//! #                          let node = &mut *successor;
//! #                          node
//! #                      };
//! #                      unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #                  };
//! #              } else {
//! #                  break;
//! #              };
//! #          }
//! #          {
//! #              let node = unsafe {
//! #                  let node = &mut *successor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #          }
//! #      }
//! #
//! #      pub fn subtree_insert_after(&mut self, new: &mut Node<'c>) {
//! #          if self.right.is_null() {
//! #              self.set_right(new);
//! #          } else {
//! #              let successor = self.successor_mut();
//! #              successor.set_left(new);
//! #          }
//! #      }
//! #
//! #      pub fn predecessor(&self) -> &'c Node<'c> {
//! #          let mut predecessor = self as *const Node<'c>;
//! #          let mut node = {
//! #              let node = unsafe {
//! #                  let node = &*predecessor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #          };
//! #
//! #          loop {
//! #              if !node.left.is_null() {
//! #                  predecessor = node.left.cast_const();
//! #                  node = {
//! #                      let node = unsafe {
//! #                          let node = &*predecessor;
//! #                          node
//! #                      };
//! #                      unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #                  };
//! #                  if !node.right.is_null() {
//! #                      predecessor = node.right.cast_const();
//! #                      node = {
//! #                          let node = unsafe {
//! #                              let node = &*predecessor;
//! #                              node
//! #                          };
//! #                          unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #                      };
//! #                  }
//! #                  break;
//! #              } else if !node.parent.is_null() {
//! #                  predecessor = node.parent.cast_const();
//! #                  node = {
//! #                      let node = unsafe {
//! #                          let node = &*predecessor;
//! #                          node
//! #                      };
//! #                      unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #                  };
//! #                  if let Some(right) = node.right() {
//! #                      if right == self {
//! #                          break;
//! #                      }
//! #                  }
//! #              }
//! #          }
//! #          node = {
//! #              let node = unsafe {
//! #                  let node = &*predecessor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #          };
//! #          node
//! #      }
//! #
//! #      pub fn predecessor_mut(&mut self) -> &'c mut Node<'c> {
//! #          let mut predecessor = UniquePointer::<Node<'c>>::from_ref_mut(self);
//! #          let mut node = predecessor.inner_mut();
//! #
//! #          loop {
//! #              if !node.left.is_null() {
//! #                  predecessor = node.left.clone();
//! #                  node = predecessor.inner_mut();
//! #                  if !node.right.is_null() {
//! #                      predecessor = node.right.clone();
//! #                      node = predecessor.inner_mut();
//! #                  }
//! #                  break;
//! #              } else if !node.parent.is_null() {
//! #                  predecessor = node.parent.clone();
//! #                  node = predecessor.inner_mut();
//! #
//! #                  if let Some(right) = node.right() {
//! #                      if right == self {
//! #                          break;
//! #                      }
//! #                  }
//! #              }
//! #          }
//! #          predecessor.inner_mut()
//! #      }
//! #
//! #      pub fn dealloc(&mut self) {
//! #          if self.refs > 0 {
//! #              self.decr_ref();
//! #          } else {
//! #              if !self.parent.is_null() {
//! #                  self.parent.drop_in_place();
//! #                  // self.parent = UniquePointer::null();
//! #              }
//! #              if !self.left.is_null() {
//! #                  self.left.drop_in_place();
//! #                  // self.left = UniquePointer::null();
//! #              }
//! #              if !self.right.is_null() {
//! #                  self.right.drop_in_place();
//! #                  // self.right = UniquePointer::null();
//! #              }
//! #              if !self.item.is_null() {
//! #                  self.item.drop_in_place();
//! #                  // self.item = UniquePointer::null();
//! #              }
//! #          }
//! #      }
//! #
//! #      pub fn swap_item(&mut self, other: &mut Self) {
//! #          unsafe {
//! #              self.item.swap(&mut other.item);
//! #          };
//! #      }
//! #
//! #      pub fn disconnect(&mut self) {
//! #          if !self.left.is_null() {
//! #              unsafe {
//! #                  let node = self.left.inner_mut();
//! #                  node.refs -= 1;
//! #              }
//! #          }
//! #          if !self.right.is_null() {
//! #              unsafe {
//! #                  let node = self.right.inner_mut();
//! #                  node.refs -= 1;
//! #              }
//! #          }
//! #          if !self.parent.is_null() {
//! #              unsafe {
//! #                  let parent = self.parent.inner_mut();
//! #                  let delete_left = if let Some(parents_left_child) = parent.left() {
//! #                      parents_left_child == self
//! #                  } else {
//! #                      false
//! #                  };
//! #                  if delete_left {
//! #                      parent.left.dealloc(true);
//! #                      parent.left = UniquePointer::null();
//! #                  } else {
//! #                      parent.right.dealloc(true);
//! #                      parent.right = UniquePointer::null();
//! #                  }
//! #                  parent.decr_ref();
//! #              }
//! #              self.parent.dealloc(true);
//! #              self.parent = UniquePointer::null();
//! #          }
//! #      }
//! #  }
//! #
//! #  pub fn subtree_delete<'c>(node: &mut Node<'c>) {
//! #      if node.leaf() {
//! #          node.decr_ref();
//! #          if node.parent.is_not_null() {
//! #              unsafe {
//! #                  let parent = node.parent.inner_mut();
//! #                  let delete_left = if let Some(parents_left_child) = parent.left() {
//! #                      parents_left_child == node
//! #                  } else {
//! #                      false
//! #                  };
//! #                  if delete_left {
//! #                      parent.left.dealloc(true);
//! #                      parent.left = UniquePointer::null();
//! #                  } else {
//! #                      parent.right.dealloc(true);
//! #                      parent.right = UniquePointer::null();
//! #                  }
//! #              }
//! #              node.parent.dealloc(true);
//! #              node.parent = UniquePointer::null();
//! #          }
//! #          node.refs.reset();
//! #          node.parent = UniquePointer::<Node<'c>>::null();
//! #          return;
//! #      } else {
//! #          let predecessor = node.predecessor_mut();
//! #          predecessor.swap_item(node);
//! #          subtree_delete(predecessor);
//! #      }
//! #  }
//! #
//! # // Node private methods
//! #  impl<'c> Node<'c> {
//! #      pub fn ptr(&self) -> UniquePointer<Node<'c>> {
//! #          let ptr = UniquePointer::copy_from_ref(self, *self.refs);
//! #          ptr
//! #      }
//! #
//! #      fn incr_ref(&mut self) {
//! #          self.refs += 1;
//! #          let mut node = self;
//! #          while !node.parent.is_null() {
//! #              unsafe {
//! #                  node = node.parent.inner_mut();
//! #                  node.refs += 1;
//! #              }
//! #          }
//! #      }
//! #
//! #      fn decr_ref(&mut self) {
//! #          self.refs -= 1;
//! #          let mut node = self;
//! #          while !node.parent.is_null() {
//! #              unsafe {
//! #                  node = node.parent.inner_mut();
//! #                  node.refs -= 1;
//! #              }
//! #          }
//! #      }
//! #
//! #      fn item_eq(&self, other: &Node<'c>) -> bool {
//! #          if self.item.addr() == other.item.addr() {
//! #              self.item.addr() == other.item.addr()
//! #          } else {
//! #              self.value() == other.value()
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<Node<'c>> for Node<'c> {
//! #      fn eq(&self, other: &Node<'c>) -> bool {
//! #          if self.item_eq(other) {
//! #              let eq = self.value().unwrap_or_default() == other.value().unwrap_or_default();
//! #              eq
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&mut Node<'c>> for Node<'c> {
//! #      fn eq(&self, other: &&mut Node<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          if self.item_eq(other) {
//! #              let eq = self.value().unwrap_or_default() == other.value().unwrap_or_default();
//! #              eq
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> Drop for Node<'c> {
//! #      fn drop(&mut self) {
//! #          self.dealloc();
//! #      }
//! #  }
//! #
//! #  impl<'c> Clone for Node<'c> {
//! #      fn clone(&self) -> Node<'c> {
//! #          let mut node = Node::nil();
//! #          node.refs = self.refs.clone();
//! #          if self.parent.is_not_null() {
//! #              node.parent = self.parent.clone();
//! #          }
//! #          if self.left.is_not_null() {
//! #              node.left = self.left.clone();
//! #          }
//! #          if self.right.is_not_null() {
//! #              node.right = self.right.clone();
//! #          }
//! #          if !self.item.is_null() {
//! #              node.item = self.item.clone();
//! #          }
//! #          node
//! #      }
//! #  }
//! #
//! #  impl<'c> AsRef<Node<'c>> for Node<'c> {
//! #      fn as_ref(&self) -> &'c Node<'c> {
//! #          unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(self) }
//! #      }
//! #  }
//! #  impl<'c> AsMut<Node<'c>> for Node<'c> {
//! #      fn as_mut(&mut self) -> &'c mut Node<'c> {
//! #          self.incr_ref();
//! #          let node = unsafe {
//! #              let node = &mut *self as *mut Node<'c>;
//! #              node
//! #          };
//! #          unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(self) }
//! #      }
//! #  }
//! #  impl<'c> std::fmt::Display for Node<'c> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(f, "{}", self.id())
//! #      }
//! #  }
//! #  impl<'c> std::fmt::Debug for Node<'c> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              [
//! #                  format!("Node@"),
//! #                  format!("{:016x}", self.addr()),
//! #                  format!("[refs={}]", *self.refs),
//! #                  if self.item.is_null() {
//! #                      format!("null")
//! #                  } else {
//! #                      format!(
//! #                          "[item={}]",
//! #                          self.value()
//! #                              .map(|value| format!("{:#?}", value))
//! #                              .unwrap_or_else(|| format!("empty"))
//! #                      )
//! #                  },
//! #                  if self.parent.is_null() {
//! #                      String::new()
//! #                  } else {
//! #                      format!(
//! #                          "(parent:{})",
//! #                          if self.parent.is_null() {
//! #                              format!("null")
//! #                          } else {
//! #                              self.parent_value()
//! #                                  .map(|parent_value| format!("{:#?}", parent_value))
//! #                                  .unwrap_or_else(|| format!("empty"))
//! #                          }
//! #                      )
//! #                  },
//! #                  if self.left.is_null() && self.right.is_null() {
//! #                      String::new()
//! #                  } else {
//! #                      format!(
//! #                          "[left:{} | right:{}]",
//! #                          if self.left.is_null() {
//! #                              format!("null")
//! #                          } else {
//! #                              self.left_value()
//! #                                  .map(|left_value| format!("{:#?}", left_value))
//! #                                  .unwrap_or_else(|| format!("empty"))
//! #                          },
//! #                          if self.right.is_null() {
//! #                              format!("null")
//! #                          } else {
//! #                              self.right_value()
//! #                                  .map(|right_value| format!("{:#?}", right_value))
//! #                                  .unwrap_or_else(|| format!("empty"))
//! #                          }
//! #                      )
//! #                  }
//! #              ]
//! #              .join("")
//! #          )
//! #      }
//! #  }
//! struct MitOpenCourseWare6006Tree<'t> {
//!     pub node_a: Node<'t>,
//!     pub node_b: Node<'t>,
//!     pub node_c: Node<'t>,
//!     pub node_d: Node<'t>,
//!     pub node_e: Node<'t>,
//!     pub node_f: Node<'t>,
//! }
//! impl<'t> MitOpenCourseWare6006Tree<'t> {
//!     pub fn initial_state() -> MitOpenCourseWare6006Tree<'t> {
//!         ///|||||||||||||||||||||||||||||||||||||||||||||\\\
//!         ///                                             \\\
//!         ///              INITIAL TREE STATE             \\\
//!         ///                                             \\\
//!         ///                     A                       \\\
//!         ///                    / \                      \\\
//!         ///                   /   \                     \\\
//!         ///                  B     C                    \\\
//!         ///                 / \                         \\\
//!         ///                /   \                        \\\
//!         ///               D     E                       \\\
//!         ///              /                              \\\
//!         ///             /                               \\\
//!         ///            F                                \\\
//!         ///                                             \\\
//!         ///                                             \\\
//!         // Scenario: Create nodes and test the equality of its items
//!         //
//!         // Given that I create disconnected nodes with values A through F
//!         let mut node_a = Node::new(Value::from("A"));
//!         let mut node_b = Node::new(Value::from("B"));
//!         let mut node_c = Node::new(Value::from("C"));
//!         let mut node_d = Node::new(Value::from("D"));
//!         let mut node_e = Node::new(Value::from("E"));
//!         let mut node_f = Node::new(Value::from("F"));
//!
//!         // Then each node has its corresponding value
//!         assert_eq!(node_a.value(), Some(Value::from("A")));
//!         assert_eq!(node_b.value(), Some(Value::from("B")));
//!         assert_eq!(node_c.value(), Some(Value::from("C")));
//!         assert_eq!(node_d.value(), Some(Value::from("D")));
//!         assert_eq!(node_e.value(), Some(Value::from("E")));
//!         assert_eq!(node_f.value(), Some(Value::from("F")));
//!
//!         /// /////////////////////////////////////////////////////////////////// ///
//!         /// Scenario: Connect nodes and check the equality of the items parents ///
//!         ///                                                                     ///
//!         /// Given that I set D as in left of B                                  ///
//!         node_b.set_left(&mut node_d);
//!         ///
//!         ///                                                                     ///
//!         /// And that I set B as in left of A before setting E as right of B     ///
//!         /// so as to test that memory references are set correctly*             ///
//!         node_a.set_left(&mut node_b);
//!         ///
//!         ///                                                                     ///
//!         /// And that I set C as left of A                                       ///
//!         node_a.set_right(&mut node_c);
//!         ///
//!         ///                                                                     ///
//!         /// And that I set E in right of B*                                     ///
//!         node_b.set_right(&mut node_e);
//!         ///
//!         ///                                                                     ///
//!         /// And that I set F in left of D                                       ///
//!         node_d.set_left(&mut node_f);
//!         ///
//!         ///                                                                     ///
//!         /// Then the parent of node B parent has value "A"                      ///
//!         assert_eq!(node_b.parent_value(), node_a.value());
//!         ///
//!         /// And the parent of node C parent has value "A"                       ///
//!         assert_eq!(node_c.parent_value(), node_a.value());
//!         ///
//!         /// And the parent of node D parent has value "B"                       ///
//!         assert_eq!(node_d.parent_value(), node_b.value());
//!         ///
//!         /// And the parent of node E parent has value "B"                       ///
//!         assert_eq!(node_e.parent_value(), node_b.value());
//!         ///
//!         ///                                                                     ///
//!         /// And the parent of node F parent has value "D"                       ///
//!         assert_eq!(node_f.parent_value(), node_d.value());
//!         ///
//!
//!         /// //////////////////////////////////////////////// ///
//!         /// Scenario: Check the equality of parent nodes     ///
//!         /// (i.e.: `impl PartialEq for Node')                ///
//!         ///                                                  ///
//!         /// Given that all nodes have been connected         ///
//!         ///                                                  ///
//!         /// Then the parent of node B is node A              ///
//!         assert_eq!(node_b.parent(), Some(&node_a));
//!         ///
//!         ///                                                  ///
//!         /// And the parent of node C is node A               ///
//!         assert_eq!(node_c.parent(), Some(&node_a));
//!         ///
//!         ///                                                  ///
//!         ///                                                  ///
//!         /// And the parent of node D is node B               ///
//!         assert_eq!(node_d.parent(), Some(&node_b));
//!         ///
//!         ///                                                  ///
//!         /// And the parent of node E is node B               ///
//!         assert_eq!(node_e.parent(), Some(&node_b));
//!         ///
//!         ///                                                  ///
//!         /// And the parent of node F is node D               ///
//!         assert_eq!(node_f.parent(), Some(&node_d));
//!         ///
//!         ///                                                  ///
//!
//!         /// ////////////////////////////////////////////////////////////////////////////////////// ///
//!         /// Scenario: Check the equality of left and right nodes                                   ///
//!         /// (i.e.: `impl PartialEq for Node')                                                      ///
//!         ///                                                                                        ///
//!         /// Given that all nodes have been connected                                               ///
//!         ///                                                                                        ///
//!         /// Then the left of node A is node B                                                      ///
//!         assert_eq!(node_a.left(), Some(&node_b));
//!         ///
//!         ///                                                                                        ///
//!         /// And the right of node A is node C                                                      ///
//!         assert_eq!(node_a.right(), Some(&node_c));
//!         ///
//!         ///                                                                                        ///
//!         /// And node A is the root node (no parent)                                                ///
//!         assert_eq!(node_a.parent(), None);
//!         ///
//!         ///                                                                                        ///
//!         ///                                                                                        ///
//!         /// And the left of node B is node D                                                       ///
//!         assert_eq!(node_b.left(), Some(&node_d));
//!         ///
//!         ///                                                                                        ///
//!         /// And the right of node B is node E                                                      ///
//!         assert_eq!(node_b.right(), Some(&node_e));
//!         ///
//!         ///                                                                                        ///
//!         /// And the parent of node B is node A                                                     ///
//!         assert_eq!(node_b.parent(), Some(&node_a));
//!         ///
//!         ///                                                                                        ///
//!         /// And node B has no grand-parent                                                         ///
//!         assert_eq!(node_b.parent().unwrap().parent(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_c.left(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_c.right(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_c.parent(), Some(&node_a));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_c.parent().unwrap().parent(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_d.left(), Some(&node_f));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_d.right(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_d.parent(), Some(&node_b));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_d.parent().unwrap().parent(), Some(&node_a));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_d.parent().unwrap().parent().unwrap().parent(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.left(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.right(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.parent(), Some(&node_d));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.parent().unwrap().parent(), Some(&node_b));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.parent().unwrap().parent().unwrap().parent(), Some(&node_a));
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.parent().unwrap().parent().unwrap().parent().unwrap().parent(), None);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_a.refs(), 9);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_b.refs(), 8);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_c.refs(), 2);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_d.refs(), 4);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_e.refs(), 2);
//!         ///
//!         ///                                                                                        ///
//!         assert_eq!(node_f.refs(), 2);
//!         ///
//!         ///                                                                                        ///
//!         let mut tree = unsafe {
//!             MitOpenCourseWare6006Tree {
//!                 node_a,
//!                 node_b,
//!                 node_c,
//!                 node_d,
//!                 node_e,
//!                 node_f,
//!             }
//!         };
//!         assert_eq!(tree.node_a.refs(), 9);
//!         assert_eq!(tree.node_b.refs(), 8);
//!         assert_eq!(tree.node_c.refs(), 2);
//!         assert_eq!(tree.node_d.refs(), 4);
//!         assert_eq!(tree.node_e.refs(), 2);
//!         assert_eq!(tree.node_f.refs(), 2);
//!
//!         tree.node_a.dealloc();
//!         tree.node_b.dealloc();
//!         tree.node_c.dealloc();
//!         tree.node_d.dealloc();
//!         tree.node_e.dealloc();
//!         tree.node_f.dealloc();
//!
//!         unsafe { std::mem::transmute::<MitOpenCourseWare6006Tree, MitOpenCourseWare6006Tree<'t>>(tree) }
//!     }
//! }
//! // test_tree_initial_state
//! MitOpenCourseWare6006Tree::initial_state();
//!
//! // test_tree_property_height
//! let tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_c.height(), 0); // leaf
//! assert_eq!(tree.node_e.height(), 0); // leaf
//! assert_eq!(tree.node_f.height(), 0); // leaf
//!
//! assert_eq!(tree.node_a.height(), 3);
//!
//! assert_eq!(tree.node_b.height(), 2);
//!
//! assert_eq!(tree.node_d.height(), 1);
//!
//!
//! // test_tree_property_depth
//! let tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_a.depth(), 0);
//!
//! assert_eq!(tree.node_b.depth(), 1);
//! assert_eq!(tree.node_c.depth(), 1);
//!
//! assert_eq!(tree.node_e.depth(), 2);
//! assert_eq!(tree.node_d.depth(), 2);
//!
//! assert_eq!(tree.node_f.depth(), 3);
//!
//!
//! // test_tree_property_leaf
//! let tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_a.leaf(), false);
//!
//! assert_eq!(tree.node_b.leaf(), false);
//! assert_eq!(tree.node_c.leaf(), true);
//!
//! assert_eq!(tree.node_d.leaf(), false);
//! assert_eq!(tree.node_e.leaf(), true);
//!
//! assert_eq!(tree.node_f.leaf(), true);
//!
//!
//! // test_tree_operation_subtree_first
//! let tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_a.subtree_first(), &tree.node_f);
//! assert_eq!(tree.node_b.subtree_first(), &tree.node_f);
//! assert_eq!(tree.node_d.subtree_first(), &tree.node_f);
//! assert_eq!(tree.node_f.subtree_first(), &tree.node_f);
//!
//! assert_eq!(tree.node_e.subtree_first(), &tree.node_e);
//! assert_eq!(tree.node_c.subtree_first(), &tree.node_c);
//!
//!
//! // test_tree_operation_successor
//! let tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_e.successor(), &tree.node_a);
//! assert_eq!(tree.node_f.successor(), &tree.node_d);
//! assert_eq!(tree.node_b.successor(), &tree.node_e);
//! assert_eq!(tree.node_d.successor(), &tree.node_b);
//! assert_eq!(tree.node_a.successor(), &tree.node_c);
//! assert_eq!(tree.node_c.successor(), &tree.node_c);
//!
//!
//! // test_tree_operation_successor_of_c
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! let mut node_g = Node::new(Value::from("G"));
//! tree.node_c.set_left(&mut node_g);
//!
//! assert_eq!(tree.node_c.successor(), &node_g);
//!
//!
//!
//! // test_tree_operation_subtree_first_mut
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_a.subtree_first_mut(), &mut tree.node_f);
//! assert_eq!(tree.node_b.subtree_first_mut(), &mut tree.node_f);
//! assert_eq!(tree.node_d.subtree_first_mut(), &mut tree.node_f);
//! assert_eq!(tree.node_f.subtree_first_mut(), &mut tree.node_f);
//!
//! assert_eq!(tree.node_e.subtree_first_mut(), &mut tree.node_e);
//! assert_eq!(tree.node_c.subtree_first_mut(), &mut tree.node_c);
//!
//!
//! // test_tree_operation_successor_mut
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_e.successor_mut(), &mut tree.node_a);
//! assert_eq!(tree.node_f.successor_mut(), &mut tree.node_d);
//! assert_eq!(tree.node_b.successor_mut(), &mut tree.node_e);
//! assert_eq!(tree.node_d.successor_mut(), &mut tree.node_b);
//! assert_eq!(tree.node_a.successor_mut(), &mut tree.node_c);
//! assert_eq!(tree.node_c.successor_mut(), &mut tree.node_c);
//!
//!
//! // test_tree_operation_successor_mut_of_c
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! let mut node_g = Node::new(Value::from("G"));
//! tree.node_c.set_left(&mut node_g);
//!
//! assert_eq!(tree.node_c.successor_mut(), &mut node_g);
//!
//!
//! // test_tree_operation_subtree_insert_after_node_when_node_left_is_null
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! let mut node_g = Node::new(Value::from("G"));
//! tree.node_c.subtree_insert_after(&mut node_g);
//!
//! assert_eq!(node_g.parent(), Some(&tree.node_c.clone()));
//!
//!
//! // test_tree_operation_subtree_insert_after_node_when_node_right_is_non_null
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! let mut node_g = Node::new(Value::from("G"));
//! tree.node_a.subtree_insert_after(&mut node_g);
//!
//! assert_eq!(node_g.parent(), tree.node_a.right());
//! assert_eq!(node_g.parent(), Some(&tree.node_c));
//!
//!
//! // test_tree_operation_predecessor
//! let tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_a.predecessor(), &tree.node_e);
//! assert_eq!(tree.node_d.predecessor(), &tree.node_f);
//! assert_eq!(tree.node_c.predecessor(), &tree.node_a);
//! assert_eq!(tree.node_e.predecessor(), &tree.node_b);
//! assert_eq!(tree.node_b.predecessor(), &tree.node_d);
//!
//!
//! // test_tree_operation_predecessor_of_g_as_right_of_e
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! let mut node_g = Node::new(Value::from("G"));
//! tree.node_e.set_right(&mut node_g);
//!
//! assert_eq!(node_g.predecessor(), &tree.node_e);
//!
//!
//! // test_tree_operation_predecessor_mut
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! assert_eq!(tree.node_a.predecessor_mut(), &mut tree.node_e);
//! assert_eq!(tree.node_d.predecessor_mut(), &mut tree.node_f);
//! assert_eq!(tree.node_c.predecessor_mut(), &mut tree.node_a);
//! assert_eq!(tree.node_e.predecessor_mut(), &mut tree.node_b);
//! assert_eq!(tree.node_b.predecessor_mut(), &mut tree.node_d);
//!
//!
//! // test_tree_operation_predecessor_mut_of_g_as_right_of_e
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! let mut node_g = Node::new(Value::from("G"));
//! tree.node_e.set_right(&mut node_g);
//!
//! assert_eq!(node_g.predecessor_mut(), &mut tree.node_e);
//!
//!
//! // test_tree_operation_swap_item
//! // Given the test tree in its initial state
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! // When I swap item of node A with item of node E
//! tree.node_a.swap_item(&mut tree.node_e);
//!
//! // Then node A has the value E
//! assert_eq!(tree.node_a.value(), Some(Value::from("E")));
//! // And node E has the value A
//! assert_eq!(tree.node_e.value(), Some(Value::from("A")));
//!
//! // And all other nodes remain with their values unmodified
//! assert_eq!(tree.node_b.value(), Some(Value::from("B")));
//! assert_eq!(tree.node_c.value(), Some(Value::from("C")));
//! assert_eq!(tree.node_d.value(), Some(Value::from("D")));
//! assert_eq!(tree.node_f.value(), Some(Value::from("F")));
//!
//!
//! // test_tree_operation_subtree_delete_leaf_nodes
//! // Given the test tree in its initial state
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! // Then node D has 3 references
//! assert_eq!(tree.node_d.refs(), 2);
//! assert_eq!(tree.node_a.refs(), 3);
//! assert_eq!(tree.node_b.refs(), 4);
//! assert_eq!(tree.node_c.refs(), 1);
//! assert_eq!(tree.node_d.refs(), 2);
//! assert_eq!(tree.node_e.refs(), 1);
//!
//! // When I subtree_delete node F
//! subtree_delete(&mut tree.node_f);
//!
//! // Then node F has no more references
//! assert_eq!(tree.node_f.refs(), 1);
//!
//! // And node F is dangling in the left of node D
//! assert_eq!(tree.node_d.left(), Some(&tree.node_f));
//!
//! // And node D has 1 reference
//! assert_eq!(tree.node_d.refs(), 1);
//!
//! // And the references of all ancestors of D are decremented
//! assert_eq!(tree.node_a.refs(), 2);
//! assert_eq!(tree.node_b.refs(), 3);
//!
//! // And the references of the other leaf nodes remains unchanged
//! assert_eq!(tree.node_c.refs(), 1);
//! assert_eq!(tree.node_e.refs(), 1);
//!
//!
//! // test_tree_operation_subtree_delete_root_node
//! // Given the test tree in its initial state
//! let mut tree = MitOpenCourseWare6006Tree::initial_state();
//!
//! // Then node A has 8 references
//! assert_eq!(tree.node_a.refs(), 3);
//! // And node B is in the left of node A
//! assert_eq!(tree.node_a.left(), Some(tree.node_b.as_ref()));
//! // And node C is in the right of node A
//! assert_eq!(tree.node_a.right(), Some(tree.node_c.as_ref()));
//!
//! // When I subtree_delete node A
//! subtree_delete(&mut tree.node_a);
//!
//! // Then node E becomes node A
//! assert_eq!(tree.node_a.value(), Some(Value::from("E")));
//!
//! // And node E (which has become A) has 2 references
//! assert_eq!(tree.node_a.refs(), 2);
//!
//! // And node B is in the left of node E (which has become A)
//! assert_eq!(tree.node_a.left(), Some(tree.node_b.as_ref()));
//!
//! // And node C is in the right of node E (which has become A)
//! assert_eq!(tree.node_a.right(), Some(tree.node_c.as_ref()));
//!
//! // And node A becomes node E
//! assert_eq!(tree.node_e.value(), Some(Value::from("A")));
//!
//! // And node A (which has become E) has no more references
//! assert_eq!(tree.node_e.refs(), 1);
//!
//! ```
//!
//! ### Testing Node
//!
//! ```
//! #  use std::borrow::Cow;
//! #  use std::convert::{AsMut, AsRef};
//! #  use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
//! #  use unique_pointer::{UniquePointer, RefCounter};
//! #  #[derive(Clone, PartialOrd, Ord, Default, PartialEq, Eq)]
//! #  pub enum Value<'c> {
//! #      #[default]
//! #      Nil,
//! #      String(Cow<'c, str>),
//! #      Byte(u8),
//! #      UInt(u64),
//! #      Int(i64),
//! #  }
//! #  impl<'c> Value<'_> {
//! #      pub fn nil() -> Value<'c> {
//! #          Value::Nil
//! #      }
//! #
//! #      pub fn is_nil(&self) -> bool {
//! #          if *self == Value::Nil {
//! #              true
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> Drop for Value<'c> {
//! #      fn drop(&mut self) {}
//! #  }
//! #
//! #  impl std::fmt::Display for Value<'_> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              match self {
//! #                  Value::Nil => "nil".to_string(),
//! #                  Value::String(h) => format!("{}", h),
//! #                  Value::Byte(h) => format!("{}", h),
//! #                  Value::UInt(h) => format!("{}", h),
//! #                  Value::Int(h) => format!("{}", h),
//! #              }
//! #          )
//! #      }
//! #  }
//! #  impl std::fmt::Debug for Value<'_> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              match self {
//! #                  Value::Nil => "nil".to_string(),
//! #                  Value::String(h) => format!("{:#?}", h),
//! #                  Value::Byte(h) => format!("{}u8", h),
//! #                  Value::UInt(h) => format!("{}u64", h),
//! #                  Value::Int(h) => format!("{}i64", h),
//! #              }
//! #          )
//! #      }
//! #  }
//! #
//! #  impl<'c> From<u8> for Value<'c> {
//! #      fn from(value: u8) -> Value<'c> {
//! #          Value::Byte(value)
//! #      }
//! #  }
//! #  impl<'c> From<u64> for Value<'c> {
//! #      fn from(value: u64) -> Value<'c> {
//! #          Value::UInt(value)
//! #      }
//! #  }
//! #  impl<'c> From<i64> for Value<'c> {
//! #      fn from(value: i64) -> Value<'c> {
//! #          Value::Int(value)
//! #      }
//! #  }
//! #  impl<'c> From<&'c str> for Value<'c> {
//! #      fn from(value: &'c str) -> Value<'c> {
//! #          Value::String(Cow::from(value))
//! #      }
//! #  }
//! #  impl<'c> From<Cow<'c, str>> for Value<'c> {
//! #      fn from(value: Cow<'c, str>) -> Value<'c> {
//! #          Value::from(value.into_owned())
//! #      }
//! #  }
//! #  impl<'c> From<&'c mut str> for Value<'c> {
//! #      fn from(value: &'c mut str) -> Value<'c> {
//! #          Value::String(Cow::<'c, str>::Borrowed(&*value))
//! #      }
//! #  }
//! #  impl<'c> From<String> for Value<'c> {
//! #      fn from(value: String) -> Value<'c> {
//! #          Value::String(Cow::from(value))
//! #      }
//! #  }
//! #  impl<'c> From<Option<String>> for Value<'c> {
//! #      fn from(value: Option<String>) -> Value<'c> {
//! #          match value {
//! #              None => Value::Nil,
//! #              Some(value) => Value::from(value),
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> AsRef<Value<'c>> for Value<'c> {
//! #      fn as_ref(&self) -> &Value<'c> {
//! #          unsafe { &*self }
//! #      }
//! #  }
//! #  impl<'c> AsMut<Value<'c>> for Value<'c> {
//! #      fn as_mut(&mut self) -> &mut Value<'c> {
//! #          unsafe { &mut *self }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&Value<'c>> for Value<'c> {
//! #      fn eq(&self, other: &&Value<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          self == other
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&mut Value<'c>> for Value<'c> {
//! #      fn eq(&self, other: &&mut Value<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          self == other
//! #      }
//! #  }
//! #
//! #
//! #  pub struct Node<'c> {
//! #      pub parent: UniquePointer<Node<'c>>,
//! #      pub left: UniquePointer<Node<'c>>,
//! #      pub right: UniquePointer<Node<'c>>,
//! #      pub item: UniquePointer<Value<'c>>,
//! #      refs: RefCounter,
//! #  }
//! #
//! #  impl<'c> Node<'c> {
//! #      pub fn nil() -> Node<'c> {
//! #          Node {
//! #              parent: UniquePointer::<Node<'c>>::null(),
//! #              left: UniquePointer::<Node<'c>>::null(),
//! #              right: UniquePointer::<Node<'c>>::null(),
//! #              item: UniquePointer::<Value<'c>>::null(),
//! #              refs: RefCounter::new(),
//! #          }
//! #      }
//! #
//! #      pub fn is_nil(&self) -> bool {
//! #          self.item.is_null()
//! #              && self.left.is_null()
//! #              && self.right.is_null()
//! #              && self.parent.is_null()
//! #              && self.refs <= 1
//! #      }
//! #
//! #      pub fn new(value: Value<'c>) -> Node<'c> {
//! #          let mut node = Node::nil();
//! #          unsafe {
//! #              node.item.write(value);
//! #          }
//! #          node
//! #      }
//! #
//! #      pub fn parent(&self) -> Option<&'c Node<'c>> {
//! #          self.parent.as_ref()
//! #      }
//! #
//! #      pub fn parent_mut(&mut self) -> Option<&'c mut Node<'c>> {
//! #          self.parent.as_mut()
//! #      }
//! #
//! #      pub fn item(&self) -> Value<'c> {
//! #          self.value().unwrap_or_default()
//! #      }
//! #
//! #      pub fn id(&self) -> String {
//! #          format!(
//! #              "{}{}",
//! #              if self.item.is_null() {
//! #                  format!("Null Node {:p}", self)
//! #              } else {
//! #                  format!("Node {}", self.item())
//! #              },
//! #              format!(" ({} referefences)", self.refs)
//! #          )
//! #      }
//! #
//! #      pub fn value(&self) -> Option<Value<'c>> {
//! #          if self.item.is_null() {
//! #              None
//! #          } else {
//! #              unsafe {
//! #                  if let Some(value) = self.item.as_ref() {
//! #                      Some(value.clone())
//! #                  } else {
//! #                      None
//! #                  }
//! #              }
//! #          }
//! #      }
//! #
//! #      pub fn parent_value(&self) -> Option<Value<'c>> {
//! #          if let Some(parent) = self.parent() {
//! #              parent.value()
//! #          } else {
//! #              None
//! #          }
//! #      }
//! #
//! #      pub fn set_left(&mut self, left: &mut Node<'c>) {
//! #          self.incr_ref();
//! #          left.parent = self.ptr();
//! #          self.left = left.ptr();
//! #          left.incr_ref();
//! #      }
//! #
//! #      pub fn set_right(&mut self, right: &mut Node<'c>) {
//! #          self.incr_ref();
//! #          right.parent = self.ptr();
//! #          self.right = right.ptr();
//! #          right.incr_ref();
//! #      }
//! #
//! #      pub fn delete_left(&mut self) {
//! #          if self.left.is_null() {
//! #              return;
//! #          }
//! #          let left = self.left.inner_mut();
//! #          left.decr_ref();
//! #          self.left.dealloc(true);
//! #          self.left = UniquePointer::null();
//! #      }
//! #
//! #      pub fn left(&self) -> Option<&'c Node<'c>> {
//! #          let left = self.left.as_ref();
//! #          left
//! #      }
//! #
//! #      pub fn left_mut(&mut self) -> Option<&'c mut Node<'c>> {
//! #          self.left.as_mut()
//! #      }
//! #
//! #      pub fn left_value(&self) -> Option<Value<'c>> {
//! #          if let Some(left) = self.left() {
//! #              left.value()
//! #          } else {
//! #              None
//! #          }
//! #      }
//! #
//! #      pub fn delete_right(&mut self) {
//! #          if self.right.is_null() {
//! #              return;
//! #          }
//! #          let right = self.right.inner_mut();
//! #          right.decr_ref();
//! #          self.right.dealloc(true);
//! #          self.right = UniquePointer::null();
//! #      }
//! #
//! #      pub fn right(&self) -> Option<&'c Node<'c>> {
//! #          self.right.as_ref()
//! #      }
//! #
//! #      pub fn right_mut(&mut self) -> Option<&'c mut Node<'c>> {
//! #          self.right.as_mut()
//! #      }
//! #
//! #      pub fn right_value(&self) -> Option<Value<'c>> {
//! #          if let Some(right) = self.right() {
//! #              right.value()
//! #          } else {
//! #              None
//! #          }
//! #      }
//! #
//! #      pub fn height(&self) -> usize {
//! #          let mut node = self;
//! #          let mut vertices = 0;
//! #
//! #          while !node.left.is_null() {
//! #              node = node.left.inner_ref();
//! #              vertices += 1;
//! #          }
//! #          vertices
//! #      }
//! #
//! #      pub fn depth(&self) -> usize {
//! #          let mut node = self;
//! #          if self.parent.is_null() {
//! #              return 0;
//! #          }
//! #          let mut vertices = 0;
//! #
//! #          while !node.parent.is_null() {
//! #              node = node.parent.inner_ref();
//! #              vertices += 1;
//! #          }
//! #          vertices
//! #      }
//! #
//! #      pub fn leaf(&self) -> bool {
//! #          self.left.is_null() && self.right.is_null()
//! #      }
//! #
//! #      pub fn addr(&self) -> usize {
//! #          (self as *const Node<'c>).addr()
//! #      }
//! #
//! #      pub fn left_addr(&self) -> usize {
//! #          self.left.addr()
//! #      }
//! #
//! #      pub fn right_addr(&self) -> usize {
//! #          self.right.addr()
//! #      }
//! #
//! #      pub fn parent_addr(&self) -> usize {
//! #          self.parent.addr()
//! #      }
//! #
//! #      pub fn refs(&self) -> usize {
//! #          *self.refs
//! #      }
//! #
//! #      pub fn subtree_first(&self) -> &'c Node<'c> {
//! #          if self.left.is_null() {
//! #              let node = self as *const Node<'c>;
//! #              return unsafe { node.as_ref().unwrap() };
//! #          }
//! #
//! #          let mut subtree_first = self.left.cast_mut();
//! #
//! #          loop {
//! #              unsafe {
//! #                  let node = &*subtree_first;
//! #                  if node.left.is_null() {
//! #                      break;
//! #                  }
//! #                  subtree_first = node.left.cast_mut()
//! #              }
//! #          }
//! #          unsafe { subtree_first.as_mut().unwrap() }
//! #      }
//! #
//! #      pub fn successor(&self) -> &'c Node<'c> {
//! #          if !self.right.is_null() {
//! #              return unsafe { self.right.as_ref().unwrap() }.subtree_first();
//! #          }
//! #
//! #          if let Some(parent) = self.parent() {
//! # // node.parent is root but node.right is null, so successor is node.subtree_first()
//! #              if parent.parent.is_null() {
//! #                  return self.subtree_first();
//! #              }
//! #          }
//! #          let mut successor = self as *const Node<'c>;
//! #          let mut node = unsafe { &*successor };
//! #          loop {
//! #              if node.left() == Some(self) {
//! #                  break;
//! #              }
//! #              if !node.parent.is_null() {
//! #                  successor = node.parent.cast_const();
//! #                  node = unsafe { &*successor };
//! #              } else {
//! #                  break;
//! #              };
//! #          }
//! #          unsafe { &*successor }
//! #      }
//! #
//! #      pub fn subtree_first_mut(&mut self) -> &'c mut Node<'c> {
//! #          if self.left.is_null() {
//! #              let node = self as *mut Node<'c>;
//! #              return {
//! #                  let node = unsafe {
//! #                      let node = &mut *node;
//! #                      node
//! #                  };
//! #                  unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #              };
//! #          }
//! #
//! #          let mut subtree_first = &mut self.left;
//! #
//! #          loop {
//! #              unsafe {
//! #                  let node = subtree_first.inner_mut();
//! #                  if node.left.is_null() {
//! #                      break;
//! #                  }
//! #                  subtree_first = &mut node.left;
//! #              }
//! #          }
//! #
//! #          subtree_first.inner_mut()
//! #      }
//! #
//! #      pub fn successor_mut(&mut self) -> &'c mut Node<'c> {
//! #          if !self.right.is_null() {
//! #              return self.right.inner_mut().subtree_first_mut();
//! #          }
//! #
//! #          if let Some(parent) = self.parent() {
//! # // node.parent is root but node.right is null, so successor is node.subtree_first_mut()
//! #              if parent.parent.is_null() {
//! #                  return self.subtree_first_mut();
//! #              }
//! #          }
//! #          let mut successor = self as *mut Node<'c>;
//! #          let mut node = {
//! #              let node = unsafe {
//! #                  let node = &mut *successor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #          };
//! #
//! #          loop {
//! #              if node.left() == Some(self) {
//! #                  break;
//! #              }
//! #              if !node.parent.is_null() {
//! #                  successor = node.parent.cast_mut();
//! #                  node = {
//! #                      let node = unsafe {
//! #                          let node = &mut *successor;
//! #                          node
//! #                      };
//! #                      unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #                  };
//! #              } else {
//! #                  break;
//! #              };
//! #          }
//! #          {
//! #              let node = unsafe {
//! #                  let node = &mut *successor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(node) }
//! #          }
//! #      }
//! #
//! #      pub fn subtree_insert_after(&mut self, new: &mut Node<'c>) {
//! #          if self.right.is_null() {
//! #              self.set_right(new);
//! #          } else {
//! #              let successor = self.successor_mut();
//! #              successor.set_left(new);
//! #          }
//! #      }
//! #
//! #      pub fn predecessor(&self) -> &'c Node<'c> {
//! #          let mut predecessor = self as *const Node<'c>;
//! #          let mut node = {
//! #              let node = unsafe {
//! #                  let node = &*predecessor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #          };
//! #
//! #          loop {
//! #              if !node.left.is_null() {
//! #                  predecessor = node.left.cast_const();
//! #                  node = {
//! #                      let node = unsafe {
//! #                          let node = &*predecessor;
//! #                          node
//! #                      };
//! #                      unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #                  };
//! #                  if !node.right.is_null() {
//! #                      predecessor = node.right.cast_const();
//! #                      node = {
//! #                          let node = unsafe {
//! #                              let node = &*predecessor;
//! #                              node
//! #                          };
//! #                          unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #                      };
//! #                  }
//! #                  break;
//! #              } else if !node.parent.is_null() {
//! #                  predecessor = node.parent.cast_const();
//! #                  node = {
//! #                      let node = unsafe {
//! #                          let node = &*predecessor;
//! #                          node
//! #                      };
//! #                      unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #                  };
//! #                  if let Some(right) = node.right() {
//! #                      if right == self {
//! #                          break;
//! #                      }
//! #                  }
//! #              }
//! #          }
//! #          node = {
//! #              let node = unsafe {
//! #                  let node = &*predecessor;
//! #                  node
//! #              };
//! #              unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(node) }
//! #          };
//! #          node
//! #      }
//! #
//! #      pub fn predecessor_mut(&mut self) -> &'c mut Node<'c> {
//! #          let mut predecessor = UniquePointer::<Node<'c>>::from_ref_mut(self);
//! #          let mut node = predecessor.inner_mut();
//! #
//! #          loop {
//! #              if !node.left.is_null() {
//! #                  predecessor = node.left.clone();
//! #                  node = predecessor.inner_mut();
//! #                  if !node.right.is_null() {
//! #                      predecessor = node.right.clone();
//! #                      node = predecessor.inner_mut();
//! #                  }
//! #                  break;
//! #              } else if !node.parent.is_null() {
//! #                  predecessor = node.parent.clone();
//! #                  node = predecessor.inner_mut();
//! #
//! #                  if let Some(right) = node.right() {
//! #                      if right == self {
//! #                          break;
//! #                      }
//! #                  }
//! #              }
//! #          }
//! #          predecessor.inner_mut()
//! #      }
//! #
//! #      pub fn dealloc(&mut self) {
//! #          if self.refs > 0 {
//! #              self.decr_ref();
//! #          } else {
//! #              if !self.parent.is_null() {
//! #                  self.parent.drop_in_place();
//! #                  // self.parent = UniquePointer::null();
//! #              }
//! #              if !self.left.is_null() {
//! #                  self.left.drop_in_place();
//! #                  // self.left = UniquePointer::null();
//! #              }
//! #              if !self.right.is_null() {
//! #                  self.right.drop_in_place();
//! #                  // self.right = UniquePointer::null();
//! #              }
//! #              if !self.item.is_null() {
//! #                  self.item.drop_in_place();
//! #                  // self.item = UniquePointer::null();
//! #              }
//! #          }
//! #      }
//! #
//! #      pub fn swap_item(&mut self, other: &mut Self) {
//! #          unsafe {
//! #              self.item.swap(&mut other.item);
//! #          };
//! #      }
//! #
//! #      pub fn disconnect(&mut self) {
//! #          if !self.left.is_null() {
//! #              unsafe {
//! #                  let node = self.left.inner_mut();
//! #                  node.refs -= 1;
//! #              }
//! #          }
//! #          if !self.right.is_null() {
//! #              unsafe {
//! #                  let node = self.right.inner_mut();
//! #                  node.refs -= 1;
//! #              }
//! #          }
//! #          if !self.parent.is_null() {
//! #              unsafe {
//! #                  let parent = self.parent.inner_mut();
//! #                  let delete_left = if let Some(parents_left_child) = parent.left() {
//! #                      parents_left_child == self
//! #                  } else {
//! #                      false
//! #                  };
//! #                  if delete_left {
//! #                      parent.left.dealloc(true);
//! #                      parent.left = UniquePointer::null();
//! #                  } else {
//! #                      parent.right.dealloc(true);
//! #                      parent.right = UniquePointer::null();
//! #                  }
//! #                  parent.decr_ref();
//! #              }
//! #              self.parent.dealloc(true);
//! #              self.parent = UniquePointer::null();
//! #          }
//! #      }
//! #  }
//! #
//! #  pub fn subtree_delete<'c>(node: &mut Node<'c>) {
//! #      if node.leaf() {
//! #          node.decr_ref();
//! #          if node.parent.is_not_null() {
//! #              unsafe {
//! #                  let parent = node.parent.inner_mut();
//! #                  let delete_left = if let Some(parents_left_child) = parent.left() {
//! #                      parents_left_child == node
//! #                  } else {
//! #                      false
//! #                  };
//! #                  if delete_left {
//! #                      parent.left.dealloc(true);
//! #                      parent.left = UniquePointer::null();
//! #                  } else {
//! #                      parent.right.dealloc(true);
//! #                      parent.right = UniquePointer::null();
//! #                  }
//! #              }
//! #              node.parent.dealloc(true);
//! #              node.parent = UniquePointer::null();
//! #          }
//! #          node.refs.reset();
//! #          node.parent = UniquePointer::<Node<'c>>::null();
//! #          return;
//! #      } else {
//! #          let predecessor = node.predecessor_mut();
//! #          predecessor.swap_item(node);
//! #          subtree_delete(predecessor);
//! #      }
//! #  }
//! #
//! # // Node private methods
//! #  impl<'c> Node<'c> {
//! #      pub fn ptr(&self) -> UniquePointer<Node<'c>> {
//! #          let ptr = UniquePointer::copy_from_ref(self, *self.refs);
//! #          ptr
//! #      }
//! #
//! #      fn incr_ref(&mut self) {
//! #          self.refs += 1;
//! #          let mut node = self;
//! #          while !node.parent.is_null() {
//! #              unsafe {
//! #                  node = node.parent.inner_mut();
//! #                  node.refs += 1;
//! #              }
//! #          }
//! #      }
//! #
//! #      fn decr_ref(&mut self) {
//! #          self.refs -= 1;
//! #          let mut node = self;
//! #          while !node.parent.is_null() {
//! #              unsafe {
//! #                  node = node.parent.inner_mut();
//! #                  node.refs -= 1;
//! #              }
//! #          }
//! #      }
//! #
//! #      fn item_eq(&self, other: &Node<'c>) -> bool {
//! #          if self.item.addr() == other.item.addr() {
//! #              self.item.addr() == other.item.addr()
//! #          } else {
//! #              self.value() == other.value()
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<Node<'c>> for Node<'c> {
//! #      fn eq(&self, other: &Node<'c>) -> bool {
//! #          if self.item_eq(other) {
//! #              let eq = self.value().unwrap_or_default() == other.value().unwrap_or_default();
//! #              eq
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> PartialEq<&mut Node<'c>> for Node<'c> {
//! #      fn eq(&self, other: &&mut Node<'c>) -> bool {
//! #          let other = unsafe { &**other };
//! #          if self.item_eq(other) {
//! #              let eq = self.value().unwrap_or_default() == other.value().unwrap_or_default();
//! #              eq
//! #          } else {
//! #              false
//! #          }
//! #      }
//! #  }
//! #
//! #  impl<'c> Drop for Node<'c> {
//! #      fn drop(&mut self) {
//! #          self.dealloc();
//! #      }
//! #  }
//! #
//! #  impl<'c> Clone for Node<'c> {
//! #      fn clone(&self) -> Node<'c> {
//! #          let mut node = Node::nil();
//! #          node.refs = self.refs.clone();
//! #          if self.parent.is_not_null() {
//! #              node.parent = self.parent.clone();
//! #          }
//! #          if self.left.is_not_null() {
//! #              node.left = self.left.clone();
//! #          }
//! #          if self.right.is_not_null() {
//! #              node.right = self.right.clone();
//! #          }
//! #          if !self.item.is_null() {
//! #              node.item = self.item.clone();
//! #          }
//! #          node
//! #      }
//! #  }
//! #
//! #  impl<'c> AsRef<Node<'c>> for Node<'c> {
//! #      fn as_ref(&self) -> &'c Node<'c> {
//! #          unsafe { std::mem::transmute::<&Node<'c>, &'c Node<'c>>(self) }
//! #      }
//! #  }
//! #  impl<'c> AsMut<Node<'c>> for Node<'c> {
//! #      fn as_mut(&mut self) -> &'c mut Node<'c> {
//! #          self.incr_ref();
//! #          let node = unsafe {
//! #              let node = &mut *self as *mut Node<'c>;
//! #              node
//! #          };
//! #          unsafe { std::mem::transmute::<&mut Node<'c>, &'c mut Node<'c>>(self) }
//! #      }
//! #  }
//! #  impl<'c> std::fmt::Display for Node<'c> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(f, "{}", self.id())
//! #      }
//! #  }
//! #  impl<'c> std::fmt::Debug for Node<'c> {
//! #      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #          write!(
//! #              f,
//! #              "{}",
//! #              [
//! #                  format!("Node@"),
//! #                  format!("{:016x}", self.addr()),
//! #                  format!("[refs={}]", *self.refs),
//! #                  if self.item.is_null() {
//! #                      format!("null")
//! #                  } else {
//! #                      format!(
//! #                          "[item={}]",
//! #                          self.value()
//! #                              .map(|value| format!("{:#?}", value))
//! #                              .unwrap_or_else(|| format!("empty"))
//! #                      )
//! #                  },
//! #                  if self.parent.is_null() {
//! #                      String::new()
//! #                  } else {
//! #                      format!(
//! #                          "(parent:{})",
//! #                          if self.parent.is_null() {
//! #                              format!("null")
//! #                          } else {
//! #                              self.parent_value()
//! #                                  .map(|parent_value| format!("{:#?}", parent_value))
//! #                                  .unwrap_or_else(|| format!("empty"))
//! #                          }
//! #                      )
//! #                  },
//! #                  if self.left.is_null() && self.right.is_null() {
//! #                      String::new()
//! #                  } else {
//! #                      format!(
//! #                          "[left:{} | right:{}]",
//! #                          if self.left.is_null() {
//! #                              format!("null")
//! #                          } else {
//! #                              self.left_value()
//! #                                  .map(|left_value| format!("{:#?}", left_value))
//! #                                  .unwrap_or_else(|| format!("empty"))
//! #                          },
//! #                          if self.right.is_null() {
//! #                              format!("null")
//! #                          } else {
//! #                              self.right_value()
//! #                                  .map(|right_value| format!("{:#?}", right_value))
//! #                                  .unwrap_or_else(|| format!("empty"))
//! #                          }
//! #                      )
//! #                  }
//! #              ]
//! #              .join("")
//! #          )
//! #      }
//! #  }
//! // test_node_nil
//! let node = Node::nil();
//!
//! assert_eq!(node.is_nil(), true);
//! assert_eq!(node.parent(), None);
//! assert_eq!(node.value(), None);
//! assert_eq!(node.left(), None);
//! assert_eq!(node.right(), None);
//! assert_eq!(node.left_value(), None);
//! assert_eq!(node.right_value(), None);
//!
//! let expected = {
//!     let node = Node::nil();
//!     node
//! };
//! assert_eq!(node, expected);
//! assert_eq!(node, Node::nil());
//!
//!
//! // test_node_new
//! let node = Node::new(Value::from("value"));
//! assert_eq!(node.is_nil(), false);
//! assert_eq!(node.parent(), None);
//! assert_eq!(node.left(), None);
//! assert_eq!(node.right(), None);
//! assert_eq!(node.left_value(), None);
//! assert_eq!(node.right_value(), None);
//!
//! assert_eq!(node.value(), Some(Value::from("value")));
//!
//! let expected = {
//!     let node = Node::new(Value::from("value"));
//!     node
//! };
//! assert_eq!(node, expected);
//! assert_eq!(node, Node::new(Value::from("value")));
//!
//!
//! // test_set_left
//! let mut node = Node::new(Value::from("value"));
//! let mut left = Node::new(Value::from("left"));
//!
//! node.set_left(&mut left);
//!
//! assert_eq!(left.parent(), Some(&node));
//!
//! assert_eq!(node.is_nil(), false);
//! assert_eq!(left.parent_value(), Some(Value::from("value")));
//! assert_eq!(left.parent(), Some(&node));
//! assert_eq!(node.value(), Some(Value::from("value")));
//! assert_eq!(node.parent(), None);
//! assert_eq!(node.left_value(), Some(Value::from("left")));
//! assert_eq!(node.refs(), 3);
//! assert_eq!(left.refs(), 2);
//! assert_eq!(node.left(), Some(&left));
//! assert_eq!(node.right_value(), None);
//! assert_eq!(node.right(), None);
//!
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut left = Node::new(Value::from("left"));
//!     node.set_left(&mut left);
//!     node
//! };
//! assert_eq!(node, expected);
//!
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut left = Node::new(Value::from("left"));
//!     node.set_left(&mut left);
//!     left
//! };
//! assert_eq!(left, expected);
//!
//! // test_set_right
//! let mut node = Node::new(Value::from("value"));
//! let mut right = Node::new(Value::from("right"));
//!
//! node.set_right(&mut right);
//!
//! assert_eq!(right.parent(), Some(&node));
//!
//! assert_eq!(node.is_nil(), false);
//! assert_eq!(right.parent_value(), Some(Value::from("value")));
//! assert_eq!(right.parent(), Some(&node));
//! assert_eq!(node.value(), Some(Value::from("value")));
//! assert_eq!(node.parent(), None);
//! assert_eq!(node.right_value(), Some(Value::from("right")));
//! assert_eq!(node.right(), Some(&right));
//! assert_eq!(node.left_value(), None);
//! assert_eq!(node.left(), None);
//!
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut left = Node::new(Value::from("right"));
//!     node.set_right(&mut left);
//!     node
//! };
//! assert_eq!(node, expected);
//!
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut left = Node::new(Value::from("right"));
//!     node.set_right(&mut left);
//!     left
//! };
//! assert_eq!(right, expected);
//!
//!
//! // test_clone_null
//! let node = Node::nil();
//! assert_eq!(node.clone(), Node::nil());
//!
//!
//! // test_clone_non_null
//! let mut node = Node::new(Value::from("value"));
//! let mut left = Node::new(Value::from("left"));
//! let mut right = Node::new(Value::from("right"));
//!
//! node.set_left(&mut left);
//! node.set_right(&mut right);
//!
//! assert_eq!(node.parent(), None);
//! assert_eq!(node.is_nil(), false);
//! assert_eq!(node.left(), Some(&left));
//! assert_eq!(node.right(), Some(&right));
//! assert_eq!(node.left_value(), Some(Value::from("left")));
//! assert_eq!(node.right_value(), Some(Value::from("right")));
//!
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut left = Node::new(Value::from("left"));
//!     let mut right = Node::new(Value::from("right"));
//!
//!     node.set_left(&mut left);
//!     node.set_right(&mut right);
//!     node
//! };
//! assert_eq!(node, expected);
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut left = Node::new(Value::from("left"));
//!     node.set_left(&mut left);
//!     left
//! };
//! assert_eq!(left, expected);
//! let expected = {
//!     let mut node = Node::new(Value::from("value"));
//!     let mut right = Node::new(Value::from("right"));
//!     node.set_right(&mut right);
//!     right
//! };
//! assert_eq!(right, expected);
//!
//! let tree = node.clone();
//! assert_eq!(node, tree);
//! ```
//!
//! ### Value Implementation
//!
//! ```
//! use std::borrow::Cow;
//! use std::convert::{AsMut, AsRef};
//! use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
//! #[derive(Clone, PartialOrd, Ord, Default, PartialEq, Eq)]
//! pub enum Value<'c> {
//!     #[default]
//!     Nil,
//!     String(Cow<'c, str>),
//!     Byte(u8),
//!     UInt(u64),
//!     Int(i64),
//! }
//! impl<'c> Value<'_> {
//!     pub fn nil() -> Value<'c> {
//!         Value::Nil
//!     }
//!
//!     pub fn is_nil(&self) -> bool {
//!         if *self == Value::Nil {
//!             true
//!         } else {
//!             false
//!         }
//!     }
//! }
//!
//! impl<'c> Drop for Value<'c> {
//!     fn drop(&mut self) {}
//! }
//!
//! impl std::fmt::Display for Value<'_> {
//!     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//!         write!(
//!             f,
//!             "{}",
//!             match self {
//!                 Value::Nil => "nil".to_string(),
//!                 Value::String(h) => format!("{}", h),
//!                 Value::Byte(h) => format!("{}", h),
//!                 Value::UInt(h) => format!("{}", h),
//!                 Value::Int(h) => format!("{}", h),
//!             }
//!         )
//!     }
//! }
//! impl std::fmt::Debug for Value<'_> {
//!     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//!         write!(
//!             f,
//!             "{}",
//!             match self {
//!                 Value::Nil => "nil".to_string(),
//!                 Value::String(h) => format!("{:#?}", h),
//!                 Value::Byte(h) => format!("{}u8", h),
//!                 Value::UInt(h) => format!("{}u64", h),
//!                 Value::Int(h) => format!("{}i64", h),
//!             }
//!         )
//!     }
//! }
//!
//! impl<'c> From<u8> for Value<'c> {
//!     fn from(value: u8) -> Value<'c> {
//!         Value::Byte(value)
//!     }
//! }
//! impl<'c> From<u64> for Value<'c> {
//!     fn from(value: u64) -> Value<'c> {
//!         Value::UInt(value)
//!     }
//! }
//! impl<'c> From<i64> for Value<'c> {
//!     fn from(value: i64) -> Value<'c> {
//!         Value::Int(value)
//!     }
//! }
//! impl<'c> From<&'c str> for Value<'c> {
//!     fn from(value: &'c str) -> Value<'c> {
//!         Value::String(Cow::from(value))
//!     }
//! }
//! impl<'c> From<Cow<'c, str>> for Value<'c> {
//!     fn from(value: Cow<'c, str>) -> Value<'c> {
//!         Value::from(value.into_owned())
//!     }
//! }
//! impl<'c> From<&'c mut str> for Value<'c> {
//!     fn from(value: &'c mut str) -> Value<'c> {
//!         Value::String(Cow::<'c, str>::Borrowed(&*value))
//!     }
//! }
//! impl<'c> From<String> for Value<'c> {
//!     fn from(value: String) -> Value<'c> {
//!         Value::String(Cow::from(value))
//!     }
//! }
//! impl<'c> From<Option<String>> for Value<'c> {
//!     fn from(value: Option<String>) -> Value<'c> {
//!         match value {
//!             None => Value::Nil,
//!             Some(value) => Value::from(value),
//!         }
//!     }
//! }
//!
//! impl<'c> AsRef<Value<'c>> for Value<'c> {
//!     fn as_ref(&self) -> &Value<'c> {
//!         unsafe { &*self }
//!     }
//! }
//! impl<'c> AsMut<Value<'c>> for Value<'c> {
//!     fn as_mut(&mut self) -> &mut Value<'c> {
//!         unsafe { &mut *self }
//!     }
//! }
//!
//! impl<'c> PartialEq<&Value<'c>> for Value<'c> {
//!     fn eq(&self, other: &&Value<'c>) -> bool {
//!         let other = unsafe { &**other };
//!         self == other
//!     }
//! }
//!
//! impl<'c> PartialEq<&mut Value<'c>> for Value<'c> {
//!     fn eq(&self, other: &&mut Value<'c>) -> bool {
//!         let other = unsafe { &**other };
//!         self == other
//!     }
//! }
//!
pub mod traits;
#[doc(inline)]
pub use traits::Pointee;
pub mod unique_pointer;
#[doc(inline)]
pub use unique_pointer::UniquePointer;
pub mod refcounter;
#[doc(inline)]
pub use refcounter::RefCounter;
pub mod smart_pointer;
#[doc(inline)]
pub use smart_pointer::SmartPointer;
