use num_traits::{ NumOps, Bounded };

use core::borrow::Borrow;
use core::iter::FromIterator;

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

#[derive(Debug, PartialEq, Eq)]
pub enum IndexVec {
    U16(Vec<u16>),
    U32(Vec<u32>)
}

impl IndexVec {
    fn get(&self, idx: usize) -> u32 {
        match self {
            IndexVec::U16(v) => v[idx] as u32,
            IndexVec::U32(v) => v[idx]
        }
    }

    fn set(&mut self, idx: usize, val: u32) {
        match self {
            IndexVec::U16(v) => { v[idx] = val as u16; },
            IndexVec::U32(v) => { v[idx] = val; }
        }
    }

    fn len(&self) -> usize {
        match self {
            IndexVec::U16(v) => v.len(),
            IndexVec::U32(v) => v.len(),
        }
    }

    fn swap(&mut self, i: usize, j: usize) {
        match self {
            IndexVec::U16(v) => { v.swap(i, j) },
            IndexVec::U32(v) => { v.swap(i, j) }
        }
    }

    #[cfg(test)]
    fn iter<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        let out: Box<dyn Iterator<Item = u32>> = match self {
            IndexVec::U16(v) => { Box::new(v.iter().map(|x| *x as u32)) },
            IndexVec::U32(v) => { Box::new(v.iter().cloned()) }
        };
        out
    }
}

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
