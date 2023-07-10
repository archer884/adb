pub trait Pairs: Iterator + Sized {
    fn pairs(self) -> PairsIter<Self>;
}

pub struct PairsIter<I: Iterator> {
    left: Option<I::Item>,
    source: I,
}

impl<I: Iterator> Iterator for PairsIter<I>
where
    I::Item: Copy,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.left.take()?;
        let right = self.source.next()?;
        self.left = Some(right);
        Some((left, right))
    }
}

impl<I: Iterator> Pairs for I {
    fn pairs(mut self) -> PairsIter<Self> {
        PairsIter {
            left: self.next(),
            source: self,
        }
    }
}
