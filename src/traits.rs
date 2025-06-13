use std::fmt::Debug;
use std::hash::Hash;

/// The [`crate::Pointee`] trait serves as a contract of sorts to ensure
/// that types used in [`crate::UniquePointer`] implement
/// Debug, because of it being considered experimental.
pub trait Pointee:  Debug {}
impl<T:  Debug> Pointee for T {}
// pub trait Pointee: Sized + Debug {}
// impl<T: Sized + Debug> Pointee for T {}
