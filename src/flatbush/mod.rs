use num_traits::{AsPrimitive, Bounded, NumOps, Zero};

use core::borrow::Borrow;
use core::iter::FromIterator;

use crate::util::IndexVec;

#[cfg(test)]
mod bench;
#[cfg(test)]
mod test;

pub trait AllowedNumber: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero
where
    Self: std::marker::Sized,
{
}

impl<T> AllowedNumber for T where T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero {}

pub struct FlatBush<T: AllowedNumber> {
    boxes: Vec<T>,
    indices: IndexVec,
    level_bounds: Vec<usize>,
    num_items: usize,
    node_size: usize,
    min_x: T,
    min_y: T,
    max_x: T,
    max_y: T,
}

pub struct FlatBushBuilder<T: AllowedNumber> {
    boxes: Vec<T>,
    node_size: usize,
    min_x: T,
    min_y: T,
    max_x: T,
    max_y: T,
}

pub const DEFAULT_NODE_SIZE: usize = 16;
pub const MIN_NODE_SIZE: usize = 2;
pub const MAX_NODE_SIZE: usize = 65535;

impl<T: AllowedNumber> FlatBushBuilder<T> {
    #[inline(always)]
    pub fn new() -> FlatBushBuilder<T> {
        FlatBushBuilder::new_with_node_size(DEFAULT_NODE_SIZE)
    }

    pub fn new_with_node_size(node_size: usize) -> FlatBushBuilder<T> {
        let node_size: usize = match node_size {
            x if x < MIN_NODE_SIZE => MIN_NODE_SIZE,
            x if x > MAX_NODE_SIZE => MAX_NODE_SIZE,
            x => x,
        };

        let min_x = T::max_value();
        let min_y = T::max_value();
        let max_x = T::min_value();
        let max_y = T::min_value();

        FlatBushBuilder { boxes: Vec::new(), node_size, min_x, min_y, max_x, max_y }
    }

    pub fn add<U: Borrow<[T; 4]>>(&mut self, new_box: U) -> usize {
        let new_box = new_box.borrow();

        self.boxes.extend_from_slice(new_box);

        if new_box[0] < self.min_x {
            self.min_x = new_box[0];
        }
        if new_box[1] < self.min_y {
            self.min_y = new_box[1];
        }
        if new_box[2] > self.max_x {
            self.max_x = new_box[2];
        }
        if new_box[3] > self.max_y {
            self.max_y = new_box[3];
        }

        (self.boxes.len() >> 2) - 1
    }

    pub fn finish(mut self) -> FlatBush<T> {
        let num_items = self.boxes.len() >> 2;

        // calculate the total number of nodes in the R-tree to allocate space for
        // and the index of each tree level
        let mut n = num_items;
        let mut num_nodes = n;
        let mut level_bounds = vec![n * 4];
        loop {
            n = ceiling_division(n, self.node_size);
            num_nodes += n;
            level_bounds.push(num_nodes * 4);
            if n == 1 {
                break;
            }
        }

        let mut indices = if num_nodes < 16384 {
            let mut v = vec![0; num_nodes];
            for i in 0..num_items {
                v[i] = i as u16;
            }
            IndexVec::U16(v)
        } else {
            let mut v = vec![0; num_nodes];
            for i in 0..num_items {
                v[i] = i as u32;
            }
            IndexVec::U32(v)
        };

        if num_items <= self.node_size {
            // only one node, skip sorting and just fill the root box
            self.boxes.push(self.min_x);
            self.boxes.push(self.min_y);
            self.boxes.push(self.max_x);
            self.boxes.push(self.max_y);
            return FlatBush {
                boxes: self.boxes,
                indices,
                level_bounds,
                num_items,
                node_size: self.node_size,
                min_x: self.min_x,
                min_y: self.min_y,
                max_x: self.max_x,
                max_y: self.max_y,
            };
        }

        let (bush_min_x, bush_min_y, bush_max_x, bush_max_y) =
            (self.min_x.as_(), self.min_y.as_(), self.max_x.as_(), self.max_y.as_());
        let width: f64 = bush_max_x - bush_min_x;
        let height: f64 = bush_max_y - bush_min_y;

        let hilbert_max = ((1 << 16) - 1) as f64;
        // map item centers into Hilbert coordinate space and calculate Hilbert values
        let mut hilbert_values: Vec<_> = (0..num_items)
            .map(|i| {
                let pos = 4 * i;
                let min_x: f64 = self.boxes[pos].as_();
                let min_y: f64 = self.boxes[pos + 1].as_();
                let max_x: f64 = self.boxes[pos + 2].as_();
                let max_y: f64 = self.boxes[pos + 3].as_();
                let x = (hilbert_max * ((min_x + max_x) / 2.0 - bush_min_x) / width).floor() as u32;
                let y =
                    (hilbert_max * ((min_y + max_y) / 2.0 - bush_min_y) / height).floor() as u32;
                hilbert(x, y)
            })
            .collect();

        // sort items by their Hilbert value (for packing later)
        sort(
            &mut hilbert_values,
            self.boxes.as_mut_slice(),
            &mut indices,
            0,
            num_items - 1,
            self.node_size,
        );

        // generate nodes at each tree level, bottom-up
        let mut pos = 0;
        for i in 0..(level_bounds.len() - 1) {
            let end = level_bounds[i];

            // generate a parent node for each block of consecutive <node_size> nodes
            while pos < end {
                let node_index = pos;

                // calculate bbox for the new node
                let mut node_min_x: T = T::max_value();
                let mut node_min_y: T = T::max_value();
                let mut node_max_x: T = T::min_value();
                let mut node_max_y: T = T::min_value();
                for _i in 0..self.node_size {
                    if pos >= end {
                        break;
                    }
                    node_min_x = min(node_min_x, self.boxes[pos]);
                    node_min_y = min(node_min_y, self.boxes[pos + 1]);
                    node_max_x = max(node_max_x, self.boxes[pos + 2]);
                    node_max_y = max(node_max_y, self.boxes[pos + 3]);
                    pos += 4;
                }

                // add the new node to the tree data
                indices.set(self.boxes.len() >> 2, node_index as u32);
                self.boxes.push(node_min_x);
                self.boxes.push(node_min_y);
                self.boxes.push(node_max_x);
                self.boxes.push(node_max_y);
            }
        }

        FlatBush {
            boxes: self.boxes,
            indices,
            level_bounds,
            num_items,
            node_size: self.node_size,
            min_x: self.min_x,
            min_y: self.min_y,
            max_x: self.max_x,
            max_y: self.max_y,
        }
    }
}

impl<T: AllowedNumber> FlatBush<T> {
    pub fn search_range<'a>(
        &'a self,
        min_x: T,
        min_y: T,
        max_x: T,
        max_y: T,
    ) -> impl Iterator<Item = usize> + 'a {
        let mut queue: Vec<usize> = vec![self.boxes.len() - 4];
        let mut pos = usize::MAX;
        let mut node_index = 0;
        let mut end: usize = 0;

        std::iter::from_fn(move || {
            if pos >= end {
                node_index = match queue.pop() {
                    Some(x) => x,
                    _ => return None,
                };
                // find the end index of the node
                end = min(
                    node_index + self.node_size * 4,
                    upper_bound(node_index, &self.level_bounds),
                );
                pos = node_index;

                if pos >= end {
                    return Some(None);
                }
            }

            let index = (self.indices.get(pos >> 2) | 0) as usize;

            // check if node bbox intersects with query bbox
            if max_x < self.boxes[pos] || // max_x < node_min_x
                max_y < self.boxes[pos + 1] || // max_y < node_min_y
                min_x > self.boxes[pos + 2] || // min_x > node_max_x
                min_y > self.boxes[pos + 3]
            // min_y > node_max_y
            {
                pos += 4;
                return Some(None);
            }

            pos += 4;
            if node_index < self.num_items * 4 {
                Some(Some(index))
            } else {
                queue.push(index); // node; add it to the search queue
                Some(None)
            }
        })
        .filter_map(|x| x)
    }

    pub fn bounds(&self) -> [T; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
    }
}

impl<T: AllowedNumber, U: Borrow<[T; 4]>> Extend<U> for FlatBushBuilder<T> {
    fn extend<I: IntoIterator<Item = U>>(&mut self, boxes: I) {
        for box_ in boxes {
            self.add(box_);
        }
    }
}

impl<T: AllowedNumber, U: Borrow<[T; 4]>> FromIterator<U> for FlatBush<T> {
    fn from_iter<I: IntoIterator<Item = U>>(boxes: I) -> Self {
        let mut builder = FlatBushBuilder::new();
        builder.extend(boxes);
        builder.finish()
    }
}

// binary search for the first value in the array bigger than the given
fn upper_bound(value: usize, arr: &[usize]) -> usize {
    let mut i = 0;
    let mut j = arr.len() - 1;
    while i < j {
        let m = (i + j) >> 1;
        if arr[m] > value {
            j = m;
        } else {
            i = m + 1;
        }
    }
    arr[i]
}

// custom quicksort that partially sorts bbox data alongside the hilbert values
fn sort<T: AllowedNumber>(
    values: &mut [u32],
    boxes: &mut [T],
    indices: &mut IndexVec,
    left: usize,
    right: usize,
    node_size: usize,
) {
    if (left / node_size) >= (right / node_size) {
        return;
    }

    let pivot = values[(left + right) >> 1];
    let mut i: isize = (left as isize) - 1;
    let mut j: isize = (right as isize) + 1;

    loop {
        loop {
            i += 1;
            if values[i as usize] >= pivot {
                break;
            }
        }
        loop {
            j -= 1;
            if values[j as usize] <= pivot {
                break;
            }
        }
        if i >= j {
            break;
        }
        swap(values, boxes, indices, i as usize, j as usize);
    }

    sort(values, boxes, indices, left, j as usize, node_size);
    sort(values, boxes, indices, (j as usize) + 1, right, node_size);
}

// swap two values and two corresponding boxes
fn swap<T: AllowedNumber>(
    values: &mut [u32],
    boxes: &mut [T],
    indices: &mut IndexVec,
    i: usize,
    j: usize,
) {
    let temp = values[i];
    values[i] = values[j];
    values[j] = temp;

    let k = 4 * i;
    let m = 4 * j;

    let a = boxes[k];
    let b = boxes[k + 1];
    let c = boxes[k + 2];
    let d = boxes[k + 3];
    boxes[k] = boxes[m];
    boxes[k + 1] = boxes[m + 1];
    boxes[k + 2] = boxes[m + 2];
    boxes[k + 3] = boxes[m + 3];
    boxes[m] = a;
    boxes[m + 1] = b;
    boxes[m + 2] = c;
    boxes[m + 3] = d;

    let e = indices.get(i);
    indices.set(i, indices.get(j));
    indices.set(j, e);
}

#[inline(always)]
fn ceiling_division(a: usize, b: usize) -> usize {
    return (a + b - 1) / b;
}

// implementing these myself to make the library work with floats even though they're not
// Ord; eventually it would be better to make OrderedFloat and num_traits play nice
#[inline(always)]
fn min<U: PartialOrd>(a: U, b: U) -> U {
    if a <= b {
        a
    } else {
        b
    }
}

#[inline(always)]
fn max<U: PartialOrd>(a: U, b: U) -> U {
    if b >= a {
        b
    } else {
        a
    }
}

// Fast Hilbert curve algorithm by http://threadlocalmutex.com/
// Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
#[allow(non_snake_case)]
fn hilbert(x: u32, y: u32) -> u32 {
    let mut a = x ^ y;
    let mut b = 0xFFFF ^ a;
    let mut c = 0xFFFF ^ (x | y);
    let mut d = x & (y ^ 0xFFFF);

    let mut A = a | (b >> 1);
    let mut B = (a >> 1) ^ a;
    let mut C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    let mut D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

    a = A;
    b = B;
    c = C;
    d = D;
    A = (a & (a >> 2)) ^ (b & (b >> 2));
    B = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
    C ^= (a & (c >> 2)) ^ (b & (d >> 2));
    D ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

    a = A;
    b = B;
    c = C;
    d = D;
    A = (a & (a >> 4)) ^ (b & (b >> 4));
    B = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
    C ^= (a & (c >> 4)) ^ (b & (d >> 4));
    D ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

    a = A;
    b = B;
    c = C;
    d = D;
    C ^= (a & (c >> 8)) ^ (b & (d >> 8));
    D ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));

    a = C ^ (C >> 1);
    b = D ^ (D >> 1);

    let mut i0 = x ^ y;
    let mut i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    ((i1 << 1) | i0) >> 0
}
