use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub(crate) struct ComputableMap<K, V, F> {
    map: RefCell<HashMap<K, Rc<V>>>,
    func: F,
}

impl<K: Clone + Eq + Hash, V, F: Fn(&K) -> V> ComputableMap<K, V, F> {
    pub fn new(func: F) -> Self {
        Self {
            map: Default::default(),
            func,
        }
    }

    /// Get the value from the map, if the value is not existed, 
    /// compute it and insert the cloned key and the computed value into the map.
    pub fn get(&self, key: &K) -> Rc<V> {
        let mut map = self.map.borrow_mut();
        let default = || Rc::new((self.func)(key));
        let inserted = map.entry(key.clone()).or_insert_with(default);
        return Rc::clone(&inserted);
    }
}

