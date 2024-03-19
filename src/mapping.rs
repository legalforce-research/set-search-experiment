use anyhow::anyhow;
use anyhow::Result;

use crate::{OrderedSet, Record};

pub struct Mapping {
    mapping: Vec<u32>,
}

impl Mapping {
    pub fn from_slice(mapping: &[u32]) -> Self {
        Self {
            mapping: mapping.to_vec(),
        }
    }

    pub fn from_records(records: &[Record<u32>], universe: u32) -> Result<Self> {
        if universe == 0 {
            return Err(anyhow!("Invalid universe."));
        }
        let mut freqs = vec![0usize; universe as usize];
        for record in records {
            for &elem in record.set.iter() {
                freqs[elem as usize] += 1;
            }
        }

        let mut elem_freq = freqs.into_iter().enumerate().collect::<Vec<_>>();
        elem_freq.sort_unstable_by(|&(_, a), &(_, b)| a.cmp(&b));

        let mut mapping = vec![0u32; universe as usize];
        for (tgt, (src, _)) in elem_freq.into_iter().enumerate() {
            mapping[src] = tgt as u32;
        }
        Ok(Self { mapping })
    }

    pub fn apply(&self, set: &OrderedSet<u32>) -> OrderedSet<u32> {
        let set = set
            .iter()
            .map(|&elem| self.mapping[elem as usize])
            .collect::<Vec<_>>();
        OrderedSet::from_unsorted(set)
    }

    pub fn universe(&self) -> u32 {
        self.mapping.len() as u32
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping() {
        let a = OrderedSet::from_sorted([0, 1, 3]).unwrap();
        let b = OrderedSet::from_sorted([0, 3]).unwrap();
        let c = OrderedSet::from_sorted([3]).unwrap();
        let records = vec![
            Record { id: 0, set: a },
            Record { id: 1, set: b },
            Record { id: 2, set: c },
        ];
        let mapping = Mapping::from_records(&records, 4).unwrap();

        let mapped = mapping.apply(&OrderedSet::from_sorted([2, 3]).unwrap());
        assert_eq!(mapped, OrderedSet::from_sorted([0, 3]).unwrap());

        let mapped = mapping.apply(&OrderedSet::from_sorted([0, 1]).unwrap());
        assert_eq!(mapped, OrderedSet::from_sorted([1, 2]).unwrap());
    }
}
