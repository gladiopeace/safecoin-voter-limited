//! This thing generates random voter groups of a given size
//! Given the set of all authorized voters (their pubkeys) it selects one randomly
//! then it picks a shift distance (some prime number less than the voter set size)
//! and iteratively selects the rest of the group by shifting that distance
//! its treating the set of voters as a ring

use crate::{pubkey::Pubkey};
use crate::hash::Hash;
use std::collections::HashMap;

use std::convert::TryInto;

pub static OPTIMAL_VOTE_GROUP_SIZE: usize = 11;
pub static SAFECOIN_NEVER_VOTER: &str = "83E5RMejo6d98FV1EAXTx5t4bvoDMoxE4DboDee3VJsu";

//#[derive(Clone, Debug, Serialize, Deserialize, AbiExample, PartialEq)]
//pub struct ArcPubkey(std::sync::Arc<Pubkey>);

#[derive(Clone, Debug, Serialize, Deserialize, AbiExample, PartialEq)]
pub struct VoteGroupGenerator {
    possible_voters: Vec<Pubkey>,
    all_distance: Vec<u32>, // a list of primes that are not factors of the possible voters group size

    group_size: usize,
}

impl VoteGroupGenerator {
    pub fn new(map: &HashMap<Pubkey, Pubkey>, size: usize) -> VoteGroupGenerator {
        let collected: Vec<_> = map.into_iter().collect();
        let mut temp = Vec::new();
        for x in collected {
            let key = x.0;
            if key.to_string() != SAFECOIN_NEVER_VOTER {
                let cloned: Pubkey = Pubkey::new_from_array(key.to_bytes());
                temp.push(cloned);
            }
        }
        let len = temp.len() as u32;
        let mut initial = Vec::new();
        initial.push(1);
        for val in [
            2, 3, 5, 7, 11, 13, 17, 23, 29, 31, 37, 41, 43, 47, 51, 53, 57, 59, 61, 67, 71, 73, 79,
            83, 87, 89, 97, 101, 103,
        ]
        .iter()
        {
            if (len > *val) && ((len % *val) != 0) {
                initial.push(*val);
            }
        }
        Self {
            possible_voters: temp,
            all_distance: initial.to_owned(),
            group_size: size,
        }
    }

    pub fn new_dummy() -> VoteGroupGenerator {
        let hm: HashMap<Pubkey, Pubkey> = HashMap::new();
        Self::new(&hm, 1)
    }

    fn ring_shift(&self, a: usize, b: usize) -> usize {
        let temp = a + b;
        temp % self.possible_voters.len()
    }

    pub fn in_group_for_hash(&self, hash: Hash, test_key: Pubkey) -> bool {
        fn hash2u64(hash_val: Hash) -> u64 {
            fn pop64(hunk: &[u8]) -> &[u8; 8] {
                hunk.try_into().expect("slice with incorrect length")
            }
            let ary = hash_val.to_bytes();
            let max = ary.len();
            if (max % 8) != 0 {
                panic!("bad hash");
            }
            let mut idx = 0;
            let mut val :u64 = 0;
            while idx < max {
                let temp = pop64(&ary[idx..(idx+8)]);
                let  valx  = u64::from_le_bytes(*temp);
                val = val ^ valx;
                idx += 8;
            }
            val
        }

        let seed = hash2u64(hash);
        self.in_group_for_seed(seed,test_key)
    }



    pub fn in_group_for_seed(&self, seed: u64, test_key: Pubkey) -> bool {
   
        let voters_len = self.possible_voters.len();
        let mut loc = (seed % voters_len as u64) as usize;
        let first_key = Pubkey::new(&self.possible_voters[loc].to_bytes());
        if test_key == first_key {
            return true;
        }
        if self.group_size > 1 {
            let choose_dist = seed % self.all_distance.len() as u64;
            let dist = self.all_distance[choose_dist as usize] as usize;
            for _ in 0..(self.group_size - 1) {
                loc = self.ring_shift(loc, dist);
                let loc_key = Pubkey::new(&self.possible_voters[loc].to_bytes());
                if test_key == loc_key {
                    println!("found {:?}", test_key);
                    return true;
                }
            }
        }
        false
    }
}

pub mod tests {
    use super::*;
    #[test]
    fn test_vgg_multi() {
        let canary = Pubkey::new_unique();
        let mut hm: HashMap<Pubkey, Pubkey> = HashMap::new();
        hm.insert(canary, Pubkey::new_unique());

        for it in 0..4 {
            let val = Pubkey::new_unique();
            hm.insert(val, Pubkey::new_unique());
            println!("insert {}", it);
        }
        let vgg = VoteGroupGenerator::new(&hm, hm.len());
        for h in hm.keys() {
            let found = vgg.in_group_for_seed(0, *h);
            assert!(found);
        }

        let not_canary = Pubkey::new_unique();
        assert_eq!(vgg.in_group_for_seed(0, not_canary), false);
    }

    #[test]
    fn test_vgg_single() {
        let canary = Pubkey::new_unique();
        let mut hm: HashMap<Pubkey, Pubkey> = HashMap::new();
        hm.insert(canary, Pubkey::new_unique());

        let vgg = VoteGroupGenerator::new(&hm, hm.len());
        for h in hm.keys() {
            let found = vgg.in_group_for_seed(0, *h);
            assert!(found);
        }

        let not_canary = Pubkey::new_unique();
        assert_eq!(vgg.in_group_for_seed(0, not_canary), false);
    }

    #[test]
    fn test_vgg_magic() {
        let magic = Pubkey::from_str(SAFECOIN_NEVER_VOTER).unwrap();
        let mut hm: HashMap<Pubkey, Pubkey> = HashMap::new();
        hm.insert(magic, Pubkey::new_unique());

        for it in 0..4 {
            let val = Pubkey::new_unique();
            hm.insert(val, Pubkey::new_unique());
            println!("insert {}", it);
        }
        let vgg = VoteGroupGenerator::new(&hm, hm.len());
        for h in hm.keys() {
            let found = vgg.in_group_for_seed(0, *h);
            let result = h.to_string() != SAFECOIN_NEVER_VOTER;
            assert_eq!(found, result);
        }
        assert_eq!(vgg.in_group_for_seed(0, magic), false);
    }
}
