# Unique Pointer

The [`unique-pointer`](https://crates.io/crates/unique-pointer) crate provides an experimental data structure
[`UniquePointer`](https://docs.rs/unique-pointer/0.1.0/unique_pointer/unique_pointer/struct.UniquePointer.html) that makes extensive use of [`unsafe`] rust to
provide a shared pointer across other data structures.

This crate is designed to be used as a building block of
data-structures and aims at being particularly useful in allowing
computer science students to implement data structures such as
linked-lists and binary trees in rust while not spending much time
tinkering with rust lifetimes.


## Crate Features

### `allow-no-debug`

Permits using `UniquePointer<T>` where `T` does not implement `std::fmt::Debug`

```
cargo add unique-pointer --allow-no-debug
```

## Linked List Example

```rust
#[derive(Clone, Debug, Hash)]
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

    assert_eq!(a.len(), 3);
}
```





## More Examples

- [binary-tree](https://en.wikipedia.org/wiki/Binary_tree)
  - [browse github](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/binary-tree)
  - [implementation](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/binary-tree/src/node.rs)
  - [node tests](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/binary-tree/tests/test_node.rs)
  - [binary tree tests](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/binary-tree/tests/test_binary_tree.rs)

- lisp ["cons"](https://en.wikipedia.org/wiki/Cons) cell
  - [browse github](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/lisp-cons-cell)
  - [cell implementation](https://github.com/gabrielfalcao/unique-pointer/blob/83b805b655fb3bd2642714a7eb214da1b71ecc4c/examples/lisp-cons-cell/src/cell.rs#L20-L262)
  - [cons/car/cdr implementation](https://github.com/gabrielfalcao/unique-pointer/blob/83b805b655fb3bd2642714a7eb214da1b71ecc4c/examples/lisp-cons-cell/src/cons.rs#L3-L63)
  - [cell tests](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/lisp-cons-cell/tests/test_cell.rs)
  - [cons/car/cdr tests](https://github.com/gabrielfalcao/unique-pointer/tree/HEAD/examples/lisp-cons-cell/tests/test_cons.rs)
