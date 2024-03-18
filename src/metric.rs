use std::cmp::Ordering;
use std::ops::RangeInclusive;

use approx::abs_diff_eq;

use crate::set::OrderedSet;

#[derive(Default, Debug, Clone, Copy)]
pub struct FilterConfig {
    pub length: bool,
    pub position: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Evaluation {
    LengthFiltered,
    PositionFiltered,
    Verified,
    Undefined,
    Accepted(f32),
}

impl Eq for Evaluation {}

impl PartialEq for Evaluation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LengthFiltered, Self::LengthFiltered) => true,
            (Self::PositionFiltered, Self::PositionFiltered) => true,
            (Self::Verified, Self::Verified) => true,
            (Self::Undefined, Self::Undefined) => true,
            (Self::Accepted(a), Self::Accepted(b)) => abs_diff_eq!(a, b),
            _ => false,
        }
    }
}

pub struct Jaccard<'a, T> {
    base: &'a OrderedSet<T>,
    overlap_factor: f32,
    length_bounds: RangeInclusive<usize>,
    config: FilterConfig,
}

impl<'a, T> Jaccard<'a, T>
where
    T: Ord + Copy,
{
    pub fn new(base: &'a OrderedSet<T>, radius: f32, config: FilterConfig) -> Self {
        let threshold = Self::threshold(radius);
        let overlap_factor = Self::overlap_factor(threshold);
        let length_bounds = Self::length_bounds(base.len(), threshold);
        Self {
            base,
            overlap_factor,
            length_bounds,
            config,
        }
    }

    pub fn update_radius(&mut self, radius: f32) {
        let threshold = Self::threshold(radius);
        self.overlap_factor = Self::overlap_factor(threshold);
        self.length_bounds = Self::length_bounds(self.base.len(), threshold);
    }

    /// Computes the similarity threshold from the radius.
    fn threshold(radius: f32) -> f32 {
        1.0 - radius.max(0.0).min(1.0)
    }

    fn overlap_factor(threshold: f32) -> f32 {
        threshold / (1. + threshold)
    }

    fn length_bounds(base_len: usize, threshold: f32) -> RangeInclusive<usize> {
        if threshold == 0.0 {
            0..=usize::MAX
        } else {
            let base_len = base_len as f32;
            let length_lower = (base_len * threshold).ceil() as usize;
            let length_upper = (base_len / threshold).floor() as usize;
            length_lower..=length_upper
        }
    }

    pub fn distance(&self, other: &OrderedSet<T>) -> Option<f32> {
        let a = self.base;
        let b = other;

        if a.is_empty() && b.is_empty() {
            return None;
        }
        if a.is_empty() || b.is_empty() {
            return Some(1.0);
        }

        let mut i = 0;
        let mut j = 0;
        let mut intersection = 0;

        while i < a.len() && j < b.len() {
            let a_i = a.get(i).unwrap();
            let b_j = b.get(j).unwrap();
            match a_i.cmp(b_j) {
                Ordering::Equal => {
                    intersection += 1;
                    i += 1;
                    j += 1;
                }
                Ordering::Less => {
                    i += 1;
                }
                Ordering::Greater => {
                    j += 1;
                }
            }
        }

        let union = a.len() + b.len() - intersection;
        Some(1.0 - (intersection as f32) / (union as f32))
    }

    pub fn evaluate(&self, other: &OrderedSet<T>) -> Evaluation {
        let a = self.base;
        let b = other;

        if a.is_empty() && b.is_empty() {
            return Evaluation::Undefined;
        }

        // radius = 1.0
        if self.overlap_factor == 0.0 {
            let dist = self.distance(b).unwrap();
            return Evaluation::Accepted(dist);
        }

        if a.is_empty() || b.is_empty() {
            return Evaluation::Verified;
        }

        let cfg = self.config;

        // 1) Length filter
        // dbg!(&self.length_bounds, b.len());
        if cfg.length && !self.length_bounds.contains(&b.len()) {
            return Evaluation::LengthFiltered;
        }

        let total_len = (a.len() + b.len()) as f32;
        let overlap_threshold = (self.overlap_factor * total_len).ceil() as usize;
        // dbg!(self.overlap_factor, overlap_threshold);

        let mut i = 0;
        let mut j = 0;
        let mut intersection = 0;

        while i < a.len() && j < b.len() {
            let a_i = a.get(i).unwrap();
            let b_j = b.get(j).unwrap();
            match a_i.cmp(b_j) {
                Ordering::Equal => {
                    intersection += 1;
                    i += 1;
                    j += 1;
                }
                Ordering::Less => {
                    i += 1;
                }
                Ordering::Greater => {
                    j += 1;
                }
            }
            // 2) Position filter
            if cfg.position {
                let a_sfx_len = a.len() - i;
                let b_sfx_len = b.len() - j;
                // dbg!(intersection, a_sfx_len, b_sfx_len);
                if intersection + a_sfx_len.min(b_sfx_len) < overlap_threshold {
                    return Evaluation::PositionFiltered;
                }
            }
        }

        if intersection < overlap_threshold {
            return Evaluation::Verified;
        }

        let union = a.len() + b.len() - intersection;
        let dist = 1.0 - (intersection as f32) / (union as f32);
        Evaluation::Accepted(dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_abs_diff_eq;

    #[test]
    fn test_jaccard() {
        let a = OrderedSet::<u32>::from_unsorted([1, 2, 3, 4, 5]);
        let b = OrderedSet::<u32>::from_unsorted([3, 4, 5, 6, 7]);
        let jaccard = Jaccard::new(&a, 1.0, FilterConfig::default());
        assert_abs_diff_eq!(jaccard.distance(&b).unwrap(), 1. - 3. / 7.);
    }

    #[test]
    fn test_length_filter_1() {
        let cfg = FilterConfig {
            length: true,
            position: false,
        };

        // J(a,b) = 1 - 4/6 = 0.333...
        let a = OrderedSet::<u32>::from_unsorted([1, 2, 3, 4, 5]);
        let b = OrderedSet::<u32>::from_unsorted([2, 3, 4, 5, 6]);

        // length_bounds = 4..=7
        assert_eq!(
            Jaccard::new(&a, 0.33, cfg).evaluate(&b),
            Evaluation::Verified
        );

        // length_bounds = 4..=7
        assert_eq!(
            Jaccard::new(&a, 0.34, cfg).evaluate(&b),
            Evaluation::Accepted(1. / 3.)
        );
    }

    #[test]
    fn test_length_filter_2() {
        let cfg = FilterConfig {
            length: true,
            position: false,
        };

        // J(a,b) = 1 - 2/3 = 0.333...
        let a = OrderedSet::<u32>::from_unsorted([1, 2]);
        let b = OrderedSet::<u32>::from_unsorted([1, 2, 3]);

        // length_bounds = 2..=2
        assert_eq!(
            Jaccard::new(&a, 0.33, cfg).evaluate(&b),
            Evaluation::LengthFiltered
        );

        // length_bounds = 2..=3
        assert_eq!(
            Jaccard::new(&a, 0.34, cfg).evaluate(&b),
            Evaluation::Accepted(1. / 3.)
        );
    }

    #[test]
    fn test_position_filter_1() {
        let cfg = FilterConfig {
            length: false,
            position: true,
        };

        // J(a,b) = 1 - 4/6 = 0.333...
        let a = OrderedSet::<u32>::from_unsorted([1, 2, 3, 4, 5]);
        let b = OrderedSet::<u32>::from_unsorted([2, 3, 4, 5, 6]);

        // overlap_threshold = 5
        // intersection = 0
        // a_sfx_len = 4
        // b_sfx_len = 5
        assert_eq!(
            Jaccard::new(&a, 0.33, cfg).evaluate(&b),
            Evaluation::PositionFiltered
        );

        // overlap_threshold = 4
        // intersection = 3
        // a_sfx_len = 1
        // b_sfx_len = 2
        assert_eq!(
            Jaccard::new(&a, 0.34, cfg).evaluate(&b),
            Evaluation::Accepted(1. / 3.)
        );
    }

    #[test]
    fn test_position_filter_2() {
        let cfg = FilterConfig {
            length: false,
            position: true,
        };

        // J(a,b) = 1 - 4/6 = 0.333...
        let a = OrderedSet::<u32>::from_unsorted([2, 3, 4, 5, 6]);
        let b = OrderedSet::<u32>::from_unsorted([2, 3, 4, 5, 7]);

        // overlap_threshold = 5
        // intersection = 4
        // a_sfx_len = 0
        // b_sfx_len = 1
        assert_eq!(
            Jaccard::new(&a, 0.33, cfg).evaluate(&b),
            Evaluation::PositionFiltered
        );

        // overlap_threshold = 4
        // intersection = 4
        // a_sfx_len = 1
        // b_sfx_len = 1
        assert_eq!(
            Jaccard::new(&a, 0.34, cfg).evaluate(&b),
            Evaluation::Accepted(1. / 3.)
        );
    }

    #[test]
    fn test_position_filter_3() {
        let cfg = FilterConfig {
            length: false,
            position: true,
        };

        // J(a,b) = 1 - 1/3 = 0.666...
        let a = OrderedSet::<u32>::from_unsorted([1]);
        let b = OrderedSet::<u32>::from_unsorted([1, 2, 3]);

        // overlap_threshold = 2
        // intersection = 0
        // a_sfx_len = 1
        // b_sfx_len = 3
        assert_eq!(
            Jaccard::new(&a, 0.66, cfg).evaluate(&b),
            Evaluation::PositionFiltered
        );

        // overlap_threshold = 1
        // intersection = 0
        // a_sfx_len = 1
        // b_sfx_len = 3
        assert_eq!(
            Jaccard::new(&a, 0.67, cfg).evaluate(&b),
            Evaluation::Accepted(2. / 3.)
        );
    }

    #[test]
    fn test_identical() {
        let cfg = FilterConfig {
            length: true,
            position: true,
        };

        let a = OrderedSet::<u32>::from_unsorted([1, 2, 3, 4, 5]);
        let b = OrderedSet::<u32>::from_unsorted([1, 2, 3, 4, 5]);

        assert_eq!(
            Jaccard::new(&a, 0.00, cfg).evaluate(&b),
            Evaluation::Accepted(0.00)
        );
        assert_eq!(
            Jaccard::new(&a, 1.00, cfg).evaluate(&b),
            Evaluation::Accepted(0.00)
        );
    }

    #[test]
    fn test_one_side_empty() {
        let cfg = FilterConfig {
            length: true,
            position: true,
        };

        let a = OrderedSet::<u32>::from_unsorted([1, 2, 3, 4, 5]);
        let b = OrderedSet::<u32>::from_unsorted([]);

        assert_eq!(
            Jaccard::new(&a, 0.00, cfg).evaluate(&b),
            Evaluation::Verified
        );
        assert_eq!(
            Jaccard::new(&a, 1.00, cfg).evaluate(&b),
            Evaluation::Accepted(1.00)
        );
    }

    #[test]
    fn test_undifined() {
        let cfg = FilterConfig {
            length: true,
            position: true,
        };

        let a = OrderedSet::<u32>::from_unsorted([]);
        let b = OrderedSet::<u32>::from_unsorted([]);

        assert_eq!(
            Jaccard::new(&a, 0.00, cfg).evaluate(&b),
            Evaluation::Undefined
        );
        assert_eq!(
            Jaccard::new(&a, 1.00, cfg).evaluate(&b),
            Evaluation::Undefined
        );
    }
}
