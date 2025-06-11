use std::fmt::Debug;

/// The `Pointee` trait serves as a contract of sorts to ensure
/// that types used in [`unique_pointer::UniquePointer`] implement
/// Debug, because of it being considered experimental.
pub trait Pointee:  Debug {}
impl<T:  Debug> Pointee for T {}
// pub trait Pointee: Sized + Debug {}
// impl<T: Sized + Debug> Pointee for T {}
