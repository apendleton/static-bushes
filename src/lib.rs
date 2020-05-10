use num_traits::{ NumOps, Bounded };

use core::borrow::Borrow;

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

// import sort from './sort';
// import range from './range';
// import within from './within';

pub struct KDBush<T: AllowedNumber> {
    node_size: usize,
    coords: Vec<T>,
    ids: IndexVec,
}



pub const DEFAULT_NODE_SIZE: usize = 64;

impl<T: AllowedNumber> KDBush<T> {
    pub fn new<U: Borrow<[T; 2]>, V: IntoIterator<Item = U>>(points: V, node_size: Option<usize>) -> KDBush<T> {
        let node_size: usize = node_size.unwrap_or(DEFAULT_NODE_SIZE);

        // store indices to the input array and coordinates in separate typed arrays
        let mut coords: Vec<T> = Vec::new();

        for point in points {
            let point = point.borrow();
            coords.push(point[0]);
            coords.push(point[1]);
        }

        let num_points = coords.len() >> 1;
        let mut ids = if num_points < 65536 {
            IndexVec::U16((0..(num_points as u16)).collect())
        } else {
            IndexVec::U32((0..(num_points as u32)).collect())
        };

        // kd-sort both arrays for efficient search (see comments in sort.js)
        sort::sort_kd(&mut ids, &mut coords, node_size, 0, num_points - 1, 0);

        KDBush { node_size, coords, ids }
    }
}
