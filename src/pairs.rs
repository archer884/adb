use std::ops::Range;

pub trait Pairs<'a, T> {
    fn pairs(self) -> PairsIter<'a, T>;
}

impl<'a, T> Pairs<'a, T> for &'a [T] {
    fn pairs(self) -> PairsIter<'a, T> {
        PairsIter::new(self)
    }
}

pub struct PairsIter<'a, T> {
    cursor: Range<usize>,
    source: &'a [T],
}

impl<'a, T> PairsIter<'a, T> {
    fn new(source: &'a [T]) -> Self {
        Self {
            cursor: 0..(source.len() - 1),
            source,
        }
    }
}

impl<'a, T> Iterator for PairsIter<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor
            .next()
            .map(|idx| (&self.source[idx], &self.source[idx + 1]))
    }
}
