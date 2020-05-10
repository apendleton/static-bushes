use num_traits::{ NumOps, Bounded };

use core::borrow::Borrow;
use core::iter::FromIterator;

use crate::util::IndexVec;

mod sort;
mod range;
mod within;
mod exact;
#[cfg(test)] mod test;
#[cfg(test)] mod bench;

pub trait AllowedNumber:
    PartialOrd + NumOps + Bounded + Copy
    where Self: std::marker::Sized {}

impl<T> AllowedNumber for T
    where T: PartialOrd + NumOps + Bounded + Copy {}

pub struct KDBush<T: AllowedNumber> {
    node_size: usize,
    coords: Vec<T>,
    ids: IndexVec,
}

pub struct KDBushBuilder<T: AllowedNumber> {
    node_size: usize,
    coords: Vec<T>,
}



pub const DEFAULT_NODE_SIZE: usize = 64;

impl<T: AllowedNumber> KDBushBuilder<T> {
    #[inline(always)]
    pub fn new() -> KDBushBuilder<T> {
        KDBushBuilder::new_with_node_size(DEFAULT_NODE_SIZE)
    }

    pub fn new_with_node_size(node_size: usize) -> KDBushBuilder<T> {
        KDBushBuilder { coords: Vec::new(), node_size }
    }

    pub fn add<U: Borrow<[T; 2]>>(&mut self, point: U) {
        let point = point.borrow();
        self.coords.push(point[0]);
        self.coords.push(point[1]);
    }

    pub fn finish(mut self) -> KDBush<T> {
        let num_points = self.coords.len() >> 1;
        let mut ids = if num_points < 65536 {
            IndexVec::U16((0..(num_points as u16)).collect())
        } else {
            IndexVec::U32((0..(num_points as u32)).collect())
        };

        // kd-sort both arrays for efficient search (see comments in sort.js)
        sort::sort_kd(&mut ids, &mut self.coords, self.node_size, 0, num_points - 1, 0);

        KDBush { node_size: self.node_size, coords: self.coords, ids }
    }
}

impl<T: AllowedNumber, U: Borrow<[T; 2]>> Extend<U> for KDBushBuilder<T> {
    fn extend<I: IntoIterator<Item = U>>(&mut self, points: I) {
        for point in points {
            self.add(point);
        }
    }
}

impl<T: AllowedNumber, U: Borrow<[T; 2]>> FromIterator<U> for KDBush<T> {
    fn from_iter<I: IntoIterator<Item = U>>(points: I) -> Self {
        let mut builder = KDBushBuilder::new();
        builder.extend(points);
        builder.finish()
    }
}
