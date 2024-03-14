use anyhow::anyhow;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderedSet<T> {
    elems: Vec<T>,
}

impl<T> OrderedSet<T>
where
    T: Ord + Copy,
{
    pub fn new() -> Self {
        Self { elems: vec![] }
    }

    pub fn from_sorted<I>(sorted: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
    {
        let mut elems = vec![];
        for elem in sorted {
            if elems.is_empty() {
                elems.push(elem);
                continue;
            }
            let last = *elems.last().unwrap();
            if last >= elem {
                return Err(anyhow!("The input must be sorted and unique."));
            }
            elems.push(elem);
        }
        Ok(Self { elems })
    }

    pub fn from_unsorted<I>(unsorted: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut elems = unsorted.into_iter().collect::<Vec<_>>();
        elems.sort_unstable_by(|a, b| a.cmp(b));
        elems.dedup();
        OrderedSet { elems }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.elems.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elems.iter()
    }

    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_sorted() {
        let set = OrderedSet::<u32>::from_sorted(vec![1, 2, 3]).unwrap();
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![&1, &2, &3]);
    }

    #[test]
    fn test_from_sorted_empty() {
        let set = OrderedSet::<u32>::from_sorted(vec![]).unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn test_from_sorted_invalid() {
        let set = OrderedSet::<u32>::from_sorted(vec![1, 2, 2, 3]);
        assert!(set.is_err());
    }

    #[test]
    fn test_from_unsorted() {
        let set = OrderedSet::<u32>::from_unsorted(vec![3, 2, 3, 1]);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![&1, &2, &3]);
    }

    #[test]
    fn test_from_unsorted_empty() {
        let set = OrderedSet::<u32>::from_unsorted(vec![]);
        assert!(set.is_empty());
    }
}
