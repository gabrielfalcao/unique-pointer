
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::fmt::{Debug, Display};

pub trait ListValue: Sized + PartialOrd + Ord + PartialEq + Eq + Clone + Debug + Display {}
impl<T: Sized + PartialOrd + Ord + PartialEq + Eq + Clone + Debug + Display> ListValue for T {}
