pub mod inverted_index;
pub mod linear_scan;
pub mod mapping;
pub mod metric;
pub mod set;
pub mod text;

use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;

use approx::abs_diff_eq;

pub use linear_scan::LinearScan;
pub use mapping::Mapping;
pub use metric::FilterConfig;
pub use set::OrderedSet;

#[derive(Debug, Clone)]
pub struct Answer {
    pub id: u32,
    pub dist: f32,
}

impl Eq for Answer {}

impl PartialEq for Answer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && abs_diff_eq!(self.dist, other.dist)
    }
}

impl Ord for Answer {
    fn cmp(&self, other: &Self) -> Ordering {
        if abs_diff_eq!(self.dist, other.dist) {
            self.id.cmp(&other.id)
        } else {
            self.dist.partial_cmp(&other.dist).unwrap()
        }
    }
}

impl PartialOrd for Answer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Record<T> {
    pub id: u32,
    pub set: OrderedSet<T>,
}
