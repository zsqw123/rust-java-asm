use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::util::ToRc;

pub struct ConstPool<Item: Eq + Hash> {
    map: HashMap<Rc<Item>, u32>,
    const_stack: Vec<Rc<Item>>,
}

impl<Item: Eq + Hash> ConstPool<Item> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            const_stack: Vec::new(),
        }
    }

    pub fn put(&mut self, item: Item) -> u32 {
        if let Some(&index) = self.map.get(&item) {
            return index;
        }
        let index = self.const_stack.len() as u32;
        let rc_item = item.rc();
        self.const_stack.push(Rc::clone(&rc_item));
        self.map.insert(rc_item, index);
        index
    }
}
