# Unique Pointer

The `unique-pointer` crate provides an experimental data structure
[`UniquePointer`] that makes extensive use of [`unsafe`] rust to
provide a shared pointer across other data structures.

This crate is designed to be used as a building block of
data-structures and aims at being particularly useful in allowing
computer science students to implement data structures such as
linked-lists and binary trees in rust while not spending much time
tinkering with rust lifetimes.

# Examples

- [binary-tree](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/binary-tree)
  - [implementation](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/binary-tree/src/node.rs)
  - [node tests](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/binary-tree/tests/test_node.rs)
  - [binary tree tests](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/binary-tree/tests/test_binary_tree.rs)

- [lisp-cons-cell](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/lisp-cons-cell)
  - [cell implementation](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/lisp-cons-cell/src/cell.rs)
  - [const/car/cdr implementation](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/lisp-cons-cell/src/cons.rs)
  - [cell tests](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/lisp-cons-cell/tests/test_cell.rs)
  - [cons/car/cdr tests](https://github.com/gabrielfalcao/unique-pointer/tree/main/examples/lisp-cons-cell/tests/test_cons.rs)
