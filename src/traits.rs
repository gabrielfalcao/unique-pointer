use std::fmt::Debug;

/// The `UniquePointee` serves as a contract of sorts to ensure that
/// types used in [`UniquePointer`] implement Debug, because of it
/// being considered experimental.
pub trait UniquePointee:  Debug {}
impl<T:  Debug> UniquePointee for T {}
// pub trait UniquePointee: Sized + Debug {}
// impl<T: Sized + Debug> UniquePointee for T {}
