use std::{cell::RefCell, ops::Deref, rc::Rc};

struct Item {
    contents: i32,
}

fn check_item(item: Option<&Item>) {}

fn main() {}
