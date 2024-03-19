use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::RangeInclusive;

use ahash::RandomState;
use anyhow::anyhow;
use anyhow::Result;
use rand::RngCore;
use rand::SeedableRng;
use rand_xoshiro::SplitMix64;

use crate::OrderedSet;

#[derive(Clone, Debug)]
pub struct FeatureExtractor {
    ngram_range: RangeInclusive<usize>,
    build_hasher: RandomState,
    universe: u32,
    seed: u64,
}

impl FeatureExtractor {
    pub fn new(
        ngram_range: RangeInclusive<usize>,
        universe: u32,
        seed: Option<u64>,
    ) -> Result<Self> {
        if universe == 0 {
            return Err(anyhow!("Invalid universe."));
        }
        if ngram_range.start() > ngram_range.end() {
            return Err(anyhow!("Invalid ngram range."));
        }
        let seed = seed.unwrap_or_else(|| rand::thread_rng().next_u64());
        let mut seeder = SplitMix64::seed_from_u64(seed);
        let build_hasher = RandomState::with_seeds(
            seeder.next_u64(),
            seeder.next_u64(),
            seeder.next_u64(),
            seeder.next_u64(),
        );
        Ok(Self {
            ngram_range,
            build_hasher,
            universe,
            seed,
        })
    }

    pub fn extract<S>(&self, tokens: &[S]) -> OrderedSet<u32>
    where
        S: AsRef<str>,
    {
        if tokens.is_empty() {
            return OrderedSet::new();
        }
        let mut features = Vec::new();
        for n in self.ngram_range.clone() {
            if tokens.len() < n {
                break;
            }
            for ngram in tokens.windows(n) {
                let hash = self.hash(ngram);
                features.push(hash);
            }
        }
        OrderedSet::from_unsorted(features)
    }

    fn hash<S>(&self, ngram: &[S]) -> u32
    where
        S: AsRef<str>,
    {
        let mut state = self.build_hasher.build_hasher();
        for gram in ngram {
            gram.as_ref().hash(&mut state);
        }
        state.finish() as u32 % self.universe
    }

    pub const fn universe(&self) -> u32 {
        self.universe
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract() {
        let extractor = FeatureExtractor::new(1..=3, u32::MAX, Some(334)).unwrap();
        let tokens = vec!["a", "b", "a", "b", "c"];
        let features = extractor.extract(&tokens);
        // a, b, c, ab, ba, bc, aba, bab, abc
        assert_eq!(features.len(), 9);
    }
}
