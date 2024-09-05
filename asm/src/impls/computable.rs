use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub type ResultRc<V, E> = Result<Rc<V>, Rc<E>>;

pub(crate) struct ComputableMap<K, V, E>(RefCell<HashMap<K, ResultRc<V, E>>>);

impl<K, V, E> Default for ComputableMap<K, V, E>
    where K: Clone + Eq + Hash {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait CacheableOwner<K, V, E, MapRef = ComputableMap<K, V, E>>
    where K: Clone + Eq + Hash {
    fn cache_map(&self) -> &MapRef;
    fn compute(&self, key: &K) -> Result<V, E>;
}

pub trait CacheAccessor<K, V, E> where K: Clone + Eq + Hash {
    fn get_with_ref(&self, key: &K) -> ResultRc<V, E>;

    /// Get the value from the map, if the value is not existed, 
    /// compute it and insert the cloned key and the computed value into the map.
    fn get(&self, key: K) -> ResultRc<V, E>;

    fn values(&self) -> Vec<ResultRc<V, E>>;
}

impl<T, K, V, E> CacheAccessor<K, V, E> for T
    where T: CacheableOwner<K, V, E>,
          K: Clone + Eq + Hash {
    fn get_with_ref(&self, key: &K) -> ResultRc<V, E> {
        let mut mut_map_ref = self.cache_map().0.borrow_mut();
        return if let Some(value) = mut_map_ref.get(key) {
            match value.as_ref() {
                Ok(v) => Ok(Rc::clone(v)),
                Err(e) => Err(Rc::clone(e)),
            }
        } else {
            let computed = match self.compute(key) {
                Ok(v) => Ok(Rc::new(v)),
                Err(e) => Err(Rc::new(e)),
            };
            let returned = computed.clone();
            mut_map_ref.insert(key.clone(), computed);
            returned
        }
    }

    fn get(&self, key: K) -> ResultRc<V, E> {
        self.get_with_ref(&key)
    }

    fn values(&self) -> Vec<ResultRc<V, E>> {
        let internal_ref_map = self.cache_map().0.borrow();
        internal_ref_map.values().cloned().collect()
    }
}
