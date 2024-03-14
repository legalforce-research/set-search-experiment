use std::collections::BinaryHeap;

use anyhow::Result;

use crate::metric::{Evaluation, FilterConfig, Jaccard};
use crate::{Answer, Mapping, OrderedSet, Record};

pub struct LinearScan {
    mapping: Mapping,
    records: Vec<Record<u32>>,
    config: FilterConfig,
}

impl LinearScan {
    pub fn from_records(records: &[Record<u32>], universe: u32) -> Result<Self> {
        let mapping = Mapping::from_records(records, universe)?;
        let records = records
            .iter()
            .map(|record| Record {
                id: record.id,
                set: mapping.apply(&record.set),
            })
            .collect::<Vec<_>>();
        Ok(Self {
            mapping,
            records,
            config: FilterConfig::default(),
        })
    }

    pub fn filter_config(mut self, config: FilterConfig) -> Self {
        self.config = config;
        self
    }

    pub fn range_query(&self, query: &OrderedSet<u32>, radius: f32) -> Vec<Answer> {
        let query = self.mapping.apply(query);
        let jaccard = Jaccard::new(&query, radius, self.config);
        let mut answers = Vec::new();
        for record in &self.records {
            if let Evaluation::Accepted(dist) = jaccard.evaluate(&record.set) {
                answers.push(Answer {
                    id: record.id,
                    dist,
                });
            }
        }
        answers.sort_unstable();
        answers
    }

    pub fn topk_query(&self, query: &OrderedSet<u32>, k: usize) -> Vec<Answer> {
        let query = self.mapping.apply(query);
        let mut jaccard = Jaccard::new(&query, 1.0, self.config);
        let mut heap = BinaryHeap::with_capacity(k);
        for record in &self.records {
            if let Evaluation::Accepted(dist) = jaccard.evaluate(&record.set) {
                if heap.len() < k {
                    heap.push(Answer {
                        id: record.id,
                        dist,
                    });
                    if heap.len() == k {
                        let max_radius = heap.peek().unwrap().dist;
                        jaccard.update_radius(max_radius);
                    }
                } else if heap.peek().unwrap().dist > dist {
                    heap.pop();
                    heap.push(Answer {
                        id: record.id,
                        dist,
                    });
                    let max_radius = heap.peek().unwrap().dist;
                    jaccard.update_radius(max_radius);
                }
            }
        }
        heap.into_sorted_vec()
    }

    pub fn all_distances(&self, query: &OrderedSet<u32>) -> Vec<Answer> {
        let query = self.mapping.apply(query);
        let jaccard = Jaccard::new(&query, 1.0, self.config);
        let mut answers = Vec::new();
        for record in &self.records {
            let dist = jaccard.distance(&record.set).unwrap_or(f32::INFINITY);
            answers.push(Answer {
                id: record.id,
                dist,
            });
        }
        answers
    }

    pub fn evaluate(&self, query: &OrderedSet<u32>, radius: f32) -> Vec<Evaluation> {
        let query = self.mapping.apply(query);
        let jaccard: Jaccard<'_, u32> = Jaccard::new(&query, radius, self.config);
        let mut evaluations = Vec::new();
        for record in &self.records {
            evaluations.push(jaccard.evaluate(&record.set));
        }
        evaluations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_distances() {
        let a = OrderedSet::from_sorted([1, 2, 3]).unwrap();
        let b = OrderedSet::from_sorted([2, 3, 4, 5]).unwrap();
        let c = OrderedSet::from_sorted([3, 4, 5, 6, 7]).unwrap();
        let records = vec![
            Record { id: 0, set: a },
            Record { id: 1, set: b },
            Record { id: 2, set: c },
        ];
        let index = LinearScan::from_records(&records, 10).unwrap();

        let query = OrderedSet::from_sorted([1, 2, 3]).unwrap();
        let answers = index.all_distances(&query);
        assert_eq!(
            answers,
            vec![
                Answer {
                    id: 0,
                    dist: 1. - 3. / 3.
                },
                Answer {
                    id: 1,
                    dist: 1. - 2. / 5.
                },
                Answer {
                    id: 2,
                    dist: 1. - 1. / 7.
                },
            ]
        );

        let query = OrderedSet::from_sorted([5, 7, 9]).unwrap();
        let answers = index.all_distances(&query);
        assert_eq!(
            answers,
            vec![
                Answer {
                    id: 0,
                    dist: 1. - 0. / 6.
                },
                Answer {
                    id: 1,
                    dist: 1. - 1. / 6.
                },
                Answer {
                    id: 2,
                    dist: 1. - 2. / 6.
                },
            ]
        );
    }
}
