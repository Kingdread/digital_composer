extern crate rand;
use std::collections::HashMap;
use std::hash::Hash;
use rand::distributions::{IndependentSample, Range};

pub struct MarkovChain<T, U> {
    chain: HashMap<T, HashMap<U, u32>>,
}

impl<T, U> MarkovChain<T, U>
where T: Copy + Eq + Hash,
      U: Copy + Eq + Hash
{
    pub fn new() -> MarkovChain<T, U> {
        MarkovChain{ chain: HashMap::new() }
    }

    pub fn mark(&mut self, prec: T, succ: U) {
        if !self.chain.contains_key(&prec) {
            self.chain.insert(prec, HashMap::new());
        };
        let mut inner_map = self.chain.get_mut(&prec).unwrap();
        let old_value = match inner_map.get(&succ) {
            Some(n) => *n,
            None => 0,
        };
        inner_map.insert(succ, 1 + old_value);
    }

    pub fn random_successor(&self, prec: T) -> Option<U> {
        let suc_map = match self.chain.get(&prec) {
            Some(m) => m,
            None => return None,
        };
        let mut high = 0;
        for i in suc_map.values() {
            high += *i;
        };
        let range = Range::new(1, high + 1);
        let mut rng = rand::thread_rng();
        let mut result = range.ind_sample(&mut rng);
        for (key, val) in suc_map.iter() {
            if *val <= result {
                return Some(*key);
            }
            result -= *val;
        }
        None
    }
}
