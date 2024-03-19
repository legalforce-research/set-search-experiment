use anyhow::Result;
use hashbrown::{HashMap, HashSet};

use crate::metric::{Evaluation, FilterConfig, Jaccard};
use crate::{Answer, Mapping, OrderedSet, Record};

const FILTER_CONFIG: FilterConfig = FilterConfig {
    length: true,
    position: true,
};

pub struct InvertedIndex {
    mapping: Mapping,
    records: Vec<Record<u32>>,
    index: HashMap<u32, Vec<u32>>,
    threshold: f32,
}

impl InvertedIndex {
    pub fn from_records(records: &[Record<u32>], universe: u32, radius: f32) -> Result<Self> {
        let threshold = Self::threshold(radius);
        let mapping = Mapping::from_records(records, universe)?;
        let records = records
            .iter()
            .map(|record| Record {
                id: record.id,
                set: mapping.apply(&record.set),
            })
            .collect::<Vec<_>>();
        let mut index = HashMap::new();
        for (i, record) in records.iter().enumerate() {
            let set_len = record.set.len() as f32;
            let pfx_len = Self::index_prefix_len(set_len, threshold);
            for &elem in record.set.iter().take(pfx_len) {
                index.entry(elem).or_insert_with(Vec::new).push(i as u32);
            }
        }
        Ok(Self {
            mapping,
            records,
            index,
            threshold,
        })
    }

    pub fn range_query(&self, query: &OrderedSet<u32>) -> Vec<Answer> {
        let query = self.mapping.apply(query);
        let set_len = query.len() as f32;
        let pfx_len = Self::query_prefix_len(set_len, self.threshold);

        let mut answers = Vec::new();
        let mut deduplicator = HashSet::new();

        let jaccard = Jaccard::new(&query, 1. - self.threshold, FILTER_CONFIG);

        for elem in query.iter().take(pfx_len) {
            if let Some(list) = self.index.get(elem) {
                for &idx in list {
                    if !deduplicator.insert(idx) {
                        continue;
                    }
                    let record = &self.records[idx as usize];
                    if let Evaluation::Accepted(dist) = jaccard.evaluate(&record.set) {
                        answers.push(Answer {
                            id: record.id,
                            dist,
                        });
                    }
                }
            }
        }

        answers.sort_unstable();
        answers
    }

    fn threshold(radius: f32) -> f32 {
        1.0 - radius.max(0.0).min(1.0)
    }

    fn index_prefix_len(set_len: f32, threshold: f32) -> usize {
        (set_len * (1. - threshold) / (1. + threshold)).floor() as usize + 1
    }

    fn query_prefix_len(set_len: f32, threshold: f32) -> usize {
        (set_len * (1. - threshold)).floor() as usize + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_search() {
        let a = OrderedSet::from_sorted([1, 2, 3]).unwrap();
        let b = OrderedSet::from_sorted([1, 2, 3, 4]).unwrap();
        let c = OrderedSet::from_sorted([2, 3, 4]).unwrap();
        let records = vec![
            Record { id: 0, set: a },
            Record { id: 1, set: b },
            Record { id: 2, set: c },
        ];

        let index = InvertedIndex::from_records(&records, 10, 0.5).unwrap();
        let query = OrderedSet::from_sorted([1, 2, 3]).unwrap();
        let answers = index.range_query(&query);
        assert_eq!(
            answers,
            vec![
                Answer {
                    id: 0,
                    dist: 1. - 3. / 3.
                },
                Answer {
                    id: 1,
                    dist: 1. - 3. / 4.
                },
                Answer {
                    id: 2,
                    dist: 1. - 2. / 4.
                },
            ]
        );

        let index = InvertedIndex::from_records(&records, 10, 0.3).unwrap();
        let query = OrderedSet::from_sorted([1, 2, 3]).unwrap();
        let answers = index.range_query(&query);
        assert_eq!(
            answers,
            vec![
                Answer {
                    id: 0,
                    dist: 1. - 3. / 3.
                },
                Answer {
                    id: 1,
                    dist: 1. - 3. / 4.
                },
            ]
        );

        let index = InvertedIndex::from_records(&records, 10, 0.1).unwrap();
        let query = OrderedSet::from_sorted([1, 2, 3]).unwrap();
        let answers = index.range_query(&query);
        assert_eq!(
            answers,
            vec![Answer {
                id: 0,
                dist: 1. - 3. / 3.
            },]
        );
    }
}
