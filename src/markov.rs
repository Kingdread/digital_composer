use std::collections::HashMap;
use std::hash::Hash;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};

#[deriving(Show)]
pub struct MarkovChain<T: Eq + Copy + Hash, U: Eq + Copy + Hash> {
    chain: HashMap<T, HashMap<U, uint>>,
}

impl<T: Eq + Copy + Hash, U: Eq + Copy + Hash> MarkovChain<T, U> {
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
        let mut high = 0u;
        for i in suc_map.values() {
            high += *i;
        };
        let range = Range::new(1u, high + 1);
        let mut rng = rand::task_rng();
        let mut result = range.ind_sample(&mut rng);
        for (key, val) in suc_map.iter() {
            if *val <= result {
                return Some(*key);
            }
            result -= *val;
        }
        return None;
    }
}
