use std::borrow::Cow;
use std::cmp::PartialEq;
use std::fmt::Debug;

use k9::assert_equal;
use unique_pointer::UniquePointer;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Value<'t> {
    String(Cow<'t, str>),
}
impl<'t> From<String> for Value<'t> {
    fn from(v: String) -> Value<'t> {
        Value::String(Cow::from(v))
    }
}
impl<'t> From<&'t str> for Value<'t> {
    fn from(v: &'t str) -> Value<'t> {
        Value::String(Cow::from(v))
    }
}

#[derive(Clone, Debug)]
pub struct Data<'t> {
    pub value: UniquePointer<Value<'t>>,
}

#[derive(Clone, Debug)]
pub struct LinkedList<T: Debug> {
    pub item: T,
    pub next: UniquePointer<LinkedList<T>>,
}
impl<T: Debug> LinkedList<T> {
    pub fn new(item: T) -> LinkedList<T> {
        LinkedList {
            item,
            next: UniquePointer::null(),
        }
    }

    pub fn append(&mut self, value: T) -> LinkedList<T> {
        let next = LinkedList::new(value);
        self.next.write_ref(&next);
        next
    }

    pub fn next(&self) -> Option<&LinkedList<T>> {
        self.next.as_ref()
    }

    pub fn len(&self) -> usize {
        let mut length = 1;

        if let Some(next) = self.next() {
            length += 1;
            length += next.len();
        }
        length
    }
}

#[test]
fn test_linked_list() {
    let mut a = LinkedList::new("a");
    let mut b = a.append("b");
    let c = b.append("c");

    assert_equal!(a.len(), 3);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryTreeNode {
    pub item: String,
    pub parent: UniquePointer<BinaryTreeNode>,
    pub left: UniquePointer<BinaryTreeNode>,
    pub right: UniquePointer<BinaryTreeNode>,
}
impl BinaryTreeNode {
    pub fn new(item: &str) -> BinaryTreeNode {
        BinaryTreeNode {
            item: String::from(item),
            parent: UniquePointer::null(),
            left: UniquePointer::null(),
            right: UniquePointer::null(),
        }
    }

    pub fn value(&self) -> &str {
        &self.item
    }

    pub fn parent(&self) -> Option<&str> {
        self.parent.as_ref().map(|parent| parent.value())
    }

    pub fn left(&self) -> Option<&str> {
        self.left.as_ref().map(|left| left.value())
    }

    pub fn right(&self) -> Option<&str> {
        self.right.as_ref().map(|right| right.value())
    }

    pub fn rotate_left(&mut self) {
        if self.parent.is_null() {
            if self.right.is_not_null() {
                self.parent = unsafe { self.right.propagate() };
                self.right = UniquePointer::null();
            }
        }
    }

    pub fn set_parent(&mut self, parent: &mut BinaryTreeNode) {
        self.parent = UniquePointer::read_only(parent);
    }

    pub fn set_left(&mut self, left: &mut BinaryTreeNode) {
        left.set_parent(self);
        self.left = UniquePointer::read_only(left);
    }

    pub fn set_right(&mut self, right: &mut BinaryTreeNode) {
        right.set_parent(self);
        self.right = UniquePointer::read_only(right);
    }
}

#[test]
fn test_binary_tree_node_partial_eq() {
    assert_equal!(BinaryTreeNode::new("A"), BinaryTreeNode::new("A"));
    assert_ne!(BinaryTreeNode::new("A"), BinaryTreeNode::new("B"));
}

#[test]
fn test_binary_tree_node_rotate_left() {
    let mut node_a = BinaryTreeNode::new("A");
    let mut node_b = BinaryTreeNode::new("B");
    let mut node_c = BinaryTreeNode::new("C");
    node_a.set_left(&mut node_b);
    node_a.set_right(&mut node_c);

    assert_equal!(node_a.value(), "A");
    assert_equal!(node_b.value(), "B");
    assert_equal!(node_c.value(), "C");

    assert_equal!(node_a.left(), Some("B"));
    assert_equal!(node_a.right(), Some("C"));

    assert_equal!(node_b.left(), None);
    assert_equal!(node_b.right(), None);

    assert_equal!(node_c.left(), None);
    assert_equal!(node_c.right(), None);

    node_a.rotate_left();
}
#[test]
fn test_unique_pointer_clone() {
    let mut data = Data {
        value: UniquePointer::from(Value::from("string")),
    };
    let mut clone = data.clone();

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_mut(), &mut Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));

    assert_equal!(clone.value.is_null(), false);
    assert_nonzero!(clone.value.addr(), "address should not be null");
    assert_equal!(clone.value.is_written(), true);
    assert_equal!(clone.value.inner_mut(), &mut Value::from("string"));
    assert_equal!(clone.value.read(), Value::from("string"));
    assert_equal!(clone.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(clone.value.as_mut(), Some(&mut Value::from("string")));

    data.value.write(Value::from("updated"));

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_mut(), &mut Value::from("updated"));
    assert_equal!(data.value.read(), Value::from("updated"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("updated")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("updated")));

    assert_equal!(clone.value.is_null(), false);
    assert_nonzero!(clone.value.addr(), "address should not be null");
    assert_equal!(clone.value.is_written(), true);
    assert_equal!(clone.value.inner_mut(), &mut Value::from("updated"));
    assert_equal!(clone.value.read(), Value::from("updated"));
    assert_equal!(clone.value.as_ref(), Some(&Value::from("updated")));
    assert_equal!(clone.value.as_mut(), Some(&mut Value::from("updated")));
}

#[test]
fn test_unique_pointer_null() {
    let data = Data {
        value: UniquePointer::null(),
    };

    assert_equal!(data.value.is_null(), true);
    assert_equal!(data.value.addr(), 0);
    assert_equal!(data.value.refs(), 1);
    assert_equal!(data.value.is_written(), false);
    assert_equal!(data.value.is_allocated(), false);
    assert_equal!(data.value.as_ref(), None);
}

#[test]
fn test_unique_pointer_write() {
    let mut data = Data {
        value: UniquePointer::null(),
    };

    data.value.write(Value::from("string"));

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_write_ref_mut() {
    let mut data = Data {
        value: UniquePointer::null(),
    };

    data.value.write_ref_mut(&mut Value::from("string"));

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_write_ref() {
    let mut data = Data {
        value: UniquePointer::null(),
    };

    data.value.write_ref(&Value::from("string"));

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_from_value() {
    let mut data = Data {
        value: UniquePointer::from(Value::from("string")),
    };

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_from_ref_clone() {
    let mut data = Data {
        value: UniquePointer::from_ref(&Value::from("string")),
    };

    assert_equal!(data.value.is_null(), false);

    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}
#[test]
fn test_unique_pointer_from_ref_copy() {
    let mut value: UniquePointer<u8> = UniquePointer::from_ref(&0xF1);

    assert_equal!(value.is_null(), false);
    assert_equal!(value.is_allocated(), true);
    assert_nonzero!(value.addr(), "address should not be null");
    assert_equal!(value.is_written(), true);
    assert_equal!(value.inner_ref(), &0xF1);
    assert_equal!(value.read(), 0xF1);
    assert_equal!(value.as_ref(), Some(&0xF1));
    assert_equal!(value.as_mut(), Some(&mut 0xF1));
}

#[test]
fn test_unique_pointer_from_mut_clone<'t>() {
    let mut value: UniquePointer<Value> = UniquePointer::from_ref_mut(&mut Value::from("string"));

    assert_equal!(value.is_null(), false);
    assert_equal!(value.is_allocated(), true);
    assert_nonzero!(value.addr(), "address should not be null");
    assert_equal!(value.is_written(), true);
    assert_equal!(value.inner_ref(), &Value::from("string"));

    assert_equal!(value.read(), Value::from("string"));
    assert_equal!(value.as_ref(), Some(&Value::from("string")));
    assert_equal!(value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_inner_mut() {
    let mut data = Data {
        value: UniquePointer::from(&mut Value::from("string")),
    };

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_mut(), &mut Value::from("string"));
    assert_equal!(data.value.refs(), 1);
    {
        let value = &*data.value;
        assert_equal!(value, &mut Value::from("string"));
        assert_equal!(data.value.refs(), 1);
    }
    assert_equal!(data.value.refs(), 1);
    {
        let value = &*data.value;
        assert_equal!(value, &Value::from("string"));
        assert_equal!(data.value.refs(), 1);
    }
    assert_equal!(data.value.refs(), 1);

    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_from_ref_mut() {
    let mut data = Data {
        value: UniquePointer::from(&mut Value::from("string")),
    };

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
    assert_nonzero!(data.value.addr(), "address should not be null");
}

#[test]
fn test_unique_pointer_from_ref() {
    let mut data = Data {
        value: UniquePointer::from(&Value::from("string")),
    };

    assert_equal!(data.value.is_null(), false);
    assert_equal!(data.value.is_allocated(), true);
    assert_nonzero!(data.value.addr(), "address should not be null");
    assert_equal!(data.value.is_written(), true);
    assert_equal!(data.value.inner_ref(), &Value::from("string"));
    assert_equal!(data.value.read(), Value::from("string"));
    assert_equal!(data.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_string_slice<'a>() {
    let string = UniquePointer::<&'a str>::from("string");
    assert_equal!(string.refs(), 1);
    assert_equal!(string.read(), "string");
    assert_equal!(string.refs(), 1);
    assert_equal!(string.read(), "string");
    assert_equal!(string.refs(), 1);
}

#[test]
fn test_unique_pointer_from_ref_outer_data_structure<'t>() {
    let data_ref = &mut Data {
        value: UniquePointer::from(Value::from("string")),
    };

    assert_equal!(data_ref.value.is_null(), false);
    assert_equal!(data_ref.value.is_allocated(), true);
    assert_nonzero!(data_ref.value.addr(), "address should not be null");
    assert_equal!(data_ref.value.is_written(), true);
    assert_equal!(&data_ref.value, &Value::from("string"));
    assert_equal!(data_ref.value.read(), Value::from("string"));
    assert_equal!(data_ref.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data_ref.value.as_mut(), Some(&mut Value::from("string")));

    let mut data_ptr = UniquePointer::<Data<'t>>::from_ref(data_ref);

    assert_equal!(data_ptr.value.is_null(), false);
    assert_equal!(data_ptr.value.is_allocated(), true);
    assert_nonzero!(data_ptr.value.addr(), "address should not be null");
    assert_equal!(data_ptr.value.is_written(), true);
    assert_equal!(data_ptr.value.inner_ref(), &Value::from("string"));
    assert_equal!(data_ptr.value.read(), Value::from("string"));
    assert_equal!(data_ptr.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data_ptr.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_unique_pointer_copy_from_ref_outer_data_structure<'t>() {
    let data_ref = &mut Data {
        value: UniquePointer::from(Value::from("string")),
    };

    assert_equal!(data_ref.value.is_null(), false);
    assert_equal!(data_ref.value.is_allocated(), true);
    assert_nonzero!(data_ref.value.addr(), "address should not be null");
    assert_equal!(data_ref.value.is_written(), true);
    assert_equal!(&data_ref.value, &Value::from("string"));
    assert_equal!(data_ref.value.read(), Value::from("string"));
    assert_equal!(data_ref.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data_ref.value.as_mut(), Some(&mut Value::from("string")));

    let mut data_ptr = UniquePointer::<Data<'t>>::copy_from_ref(data_ref, 0);

    assert_nonzero!(
        data_ptr.inner_ref().value.addr(),
        "address should not be null"
    );
    assert_equal!(data_ptr.inner_ref().value.is_null(), false);
    assert_equal!(data_ptr.inner_ref().value.is_allocated(), true);
    assert_equal!(data_ptr.inner_ref().value.is_written(), true);
    assert_equal!(&data_ptr.inner_ref().value, &Value::from("string"));
    assert_equal!(data_ptr.inner_ref().value.read(), Value::from("string"));
    assert_equal!(
        data_ptr.inner_ref().value.as_ref(),
        Some(&Value::from("string"))
    );
    assert_equal!(
        data_ptr.inner_mut().value.as_mut(),
        Some(&mut Value::from("string"))
    );
}

#[test]
fn test_unique_pointer_copy_from_ref_deref_outer_data_structure<'t>() {
    let data_ref = &mut Data {
        value: UniquePointer::from(Value::from("string")),
    };

    assert_equal!(data_ref.value.is_null(), false);
    assert_equal!(data_ref.value.is_allocated(), true);
    assert_nonzero!(data_ref.value.addr(), "address should not be null");
    assert_equal!(data_ref.value.is_written(), true);
    assert_equal!(&data_ref.value, &Value::from("string"));
    assert_equal!(data_ref.value.read(), Value::from("string"));
    assert_equal!(data_ref.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data_ref.value.as_mut(), Some(&mut Value::from("string")));

    let mut data_ptr = UniquePointer::<Data<'t>>::copy_from_ref(data_ref, 0);

    assert_nonzero!(data_ptr.value.addr(), "address should not be null");
    assert_equal!(data_ptr.value.is_null(), false);
    assert_equal!(data_ptr.value.is_allocated(), true);
    assert_equal!(data_ptr.value.is_written(), true);
    assert_equal!(&data_ptr.value, &Value::from("string"));
    assert_equal!(data_ptr.value.read(), Value::from("string"));
    assert_equal!(data_ptr.value.as_ref(), Some(&Value::from("string")));
    assert_equal!(data_ptr.value.as_mut(), Some(&mut Value::from("string")));
}

#[test]
fn test_dealloc<'t>() {
    let mut up = UniquePointer::from(Value::from("string"));

    assert_equal!(up.refs(), 1);
    let up2 = unsafe { up.propagate() };
    assert_equal!(up.refs(), 2);
    drop(up2);
    assert_equal!(up.refs(), 1);
    unsafe {
        up.propagate();
    }
    assert_equal!(up.refs(), 1);
    unsafe {
        up.propagate().dealloc(true);
    }
    assert_equal!(up.refs(), 0);
    up.dealloc(true);
    assert_equal!(up.refs(), 0);
    assert_equal!(up.is_null(), true);
}

#[macro_export]
macro_rules! assert_nonzero {
    ($value:expr, $desc:literal) => {{
        k9::assert_greater_than!($value, 0, $desc);
    }};
    ($value:expr) => {{
        k9::assert_greater_than!($value, 0);
    }};
}
