use rayon::{
    iter::plumbing::{bridge_unindexed, Folder, UnindexedConsumer, UnindexedProducer},
    prelude::*,
};

use crate::{Generator, JumpableGenerator};

pub struct RngIter<T> {
    rng: T,
}

impl<'a, T> Iterator for RngIter<'a, T>
where
    T: 'a + Generator + JumpableGenerator,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(&mut self.rng)
    }
}

pub struct ParRngIter<T> {
    rng: T,
}

impl<T> UnindexedProducer for ParRngIter<T>
where
    T: Generator + JumpableGenerator + Send,
{
    type Item = T;

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        todo!()
    }

    fn split(self) -> (Self, Option<Self>) {
        todo!()
    }
}

impl<T> ParallelIterator for ParRngIter<T>
where
    T: Generator + JumpableGenerator + Send,
{
    type Item = T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}
