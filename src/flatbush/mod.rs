use num_traits::{ NumOps, AsPrimitive, Bounded, Zero };

#[cfg(test)] mod test;
#[cfg(test)] mod bench;

#[derive(Debug, PartialEq, Eq)]
enum IndexVec {
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

    #[cfg(test)]
    fn iter<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        let out: Box<dyn Iterator<Item = u32>> = match self {
            IndexVec::U16(v) => { Box::new(v.iter().map(|x| *x as u32)) },
            IndexVec::U32(v) => { Box::new(v.iter().cloned()) }
        };
        out
    }
}

pub struct Flatbush<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero> {
    boxes: Vec<T>,
    indices: IndexVec,
    level_bounds: Vec<usize>,
    num_items: usize,
    node_size: usize,
    pos: usize,
    min_x: T,
    min_y: T,
    max_x: T,
    max_y: T,
}

pub struct FlatbushBuilder<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero> {
    bush: Flatbush<T>
}



pub const DEFAULT_NODE_SIZE: usize = 16;
pub const MIN_NODE_SIZE: usize = 2;
pub const MAX_NODE_SIZE: usize = 65535;

impl<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero> FlatbushBuilder<T> {
    pub fn new(num_items: usize, node_size: Option<usize>) -> FlatbushBuilder<T> {
        let node_size: usize = match node_size {
            None => DEFAULT_NODE_SIZE,
            Some(x) if x < MIN_NODE_SIZE => MIN_NODE_SIZE,
            Some(x) if x > MAX_NODE_SIZE => MAX_NODE_SIZE,
            Some(x) => x,
        };

        // calculate the total number of nodes in the R-tree to allocate space for
        // and the index of each tree level (used in search later)
        let mut n = num_items;
        let mut num_nodes = n;
        let mut level_bounds = vec![n * 4];
        loop {
            n = ceiling_division(n, node_size);
            num_nodes += n;
            level_bounds.push(num_nodes * 4);
            if n == 1 { break; }
        }


        let boxes_len = num_nodes * 4;
        let mut boxes = Vec::with_capacity(boxes_len);
        boxes.resize(boxes_len, T::zero());

        let indices = if num_nodes < 16384 {
            IndexVec::U16(vec![0u16; num_nodes])
        } else {
            IndexVec::U32(vec![0u32; num_nodes])
        };

        let pos = 0;
        let min_x = T::max_value();
        let min_y = T::max_value();
        let max_x = T::min_value();
        let max_y = T::min_value();

        let bush: Flatbush<T> = Flatbush { boxes, indices, level_bounds, num_items, node_size, pos, min_x, min_y, max_x, max_y };
        FlatbushBuilder { bush }

        // a priority queue for k-nearest-neighbors queries
        // self.queue = new FlatQueue();
    }

    pub fn add(&mut self, min_x: T, min_y: T, max_x: T, max_y: T) -> usize {
        let bush = &mut self.bush;

        let index = bush.pos >> 2;
        bush.indices.set(index, index as u32);
        bush.boxes[bush.pos] = min_x;
        bush.boxes[bush.pos + 1] = min_y;
        bush.boxes[bush.pos + 2] = max_x;
        bush.boxes[bush.pos + 3] = max_y;
        bush.pos += 4;

        if min_x < bush.min_x { bush.min_x = min_x; }
        if min_y < bush.min_y { bush.min_y = min_y; }
        if max_x > bush.max_x { bush.max_x = max_x; }
        if max_y > bush.max_y { bush.max_y = max_y; }

        index
    }

    pub fn finish(self) -> Flatbush<T> {
        let mut bush = self.bush;

        if bush.pos >> 2 != bush.num_items {
            panic!("Added {} items when expected {}.", bush.pos >> 2, bush.num_items);
        }

        if bush.num_items <= bush.node_size {
            // only one node, skip sorting and just fill the root box
            bush.boxes[bush.pos] = bush.min_x;
            bush.boxes[bush.pos + 1] = bush.min_y;
            bush.boxes[bush.pos + 2] = bush.max_x;
            bush.boxes[bush.pos + 3] = bush.max_y;
            bush.pos += 4;
            return bush;
        }

        let (bush_min_x, bush_min_y, bush_max_x, bush_max_y) = (bush.min_x.as_(), bush.min_y.as_(), bush.max_x.as_(), bush.max_y.as_());
        let width: f64 = bush_max_x - bush_min_x;
        let height: f64 = bush_max_y - bush_min_y;
        let mut hilbert_values = vec![0u32; bush.num_items];
        let hilbert_max = ((1 << 16) - 1) as f64;

        // map item centers into Hilbert coordinate space and calculate Hilbert values
        for i in 0..bush.num_items {
            let pos = 4 * i;
            let min_x: f64 = bush.boxes[pos].as_();
            let min_y: f64 = bush.boxes[pos + 1].as_();
            let max_x: f64 = bush.boxes[pos + 2].as_();
            let max_y: f64 = bush.boxes[pos + 3].as_();
            let x = (hilbert_max * ((min_x + max_x) / 2.0 - bush_min_x) / width).floor() as u32;
            let y = (hilbert_max * ((min_y + max_y) / 2.0 - bush_min_y) / height).floor() as u32;
            hilbert_values[i] = hilbert(x, y);
        }

        // sort items by their Hilbert value (for packing later)
        sort(&mut hilbert_values, bush.boxes.as_mut_slice(), &mut bush.indices, 0, bush.num_items - 1, bush.node_size);

        // generate nodes at each tree level, bottom-up
        let mut pos = 0;
        for i in 0..(bush.level_bounds.len() - 1) {
            let end = bush.level_bounds[i];

            // generate a parent node for each block of consecutive <node_size> nodes
            while pos < end {
                let node_index = pos;

                // calculate bbox for the new node
                let mut node_min_x: T = T::max_value();
                let mut node_min_y: T = T::max_value();
                let mut node_max_x: T = T::min_value();
                let mut node_max_y: T = T::min_value();
                for _i in 0..bush.node_size {
                    if pos >= end { break; }
                    node_min_x = min(node_min_x, bush.boxes[pos]);
                    node_min_y = min(node_min_y, bush.boxes[pos + 1]);
                    node_max_x = max(node_max_x, bush.boxes[pos + 2]);
                    node_max_y = max(node_max_y, bush.boxes[pos + 3]);
                    pos += 4;
                }

                // add the new node to the tree data
                bush.indices.set(bush.pos >> 2, node_index as u32);
                bush.boxes[bush.pos] = node_min_x;
                bush.boxes[bush.pos + 1] = node_min_y;
                bush.boxes[bush.pos + 2] = node_max_x;
                bush.boxes[bush.pos + 3] = node_max_y;
                bush.pos += 4;
            }
        }

        bush
    }
}

impl<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero> Flatbush<T> {
    pub fn search_range<'a>(&'a self, min_x: T, min_y: T, max_x: T, max_y: T) -> impl Iterator<Item = usize> + 'a {
        let mut queue: Vec<usize> = vec![self.boxes.len() - 4];
        let mut pos = usize::MAX;
        let mut node_index = 0;
        let mut end: usize = 0;

        std::iter::from_fn(move || {
            if pos >= end {
                node_index = match queue.pop() {
                    Some(x) => x,
                    _ => return None
                };
                // find the end index of the node
                end = min(node_index + self.node_size * 4, upper_bound(node_index, &self.level_bounds));
                pos = node_index;

                if pos >= end {
                    return Some(None);
                }
            }

            let index = (self.indices.get(pos >> 2) | 0) as usize;

            // check if node bbox intersects with query bbox
            if  max_x < self.boxes[pos] || // max_x < node_min_x
                max_y < self.boxes[pos + 1] || // max_y < node_min_y
                min_x > self.boxes[pos + 2] || // min_x > node_max_x
                min_y > self.boxes[pos + 3] // min_y > node_max_y
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
        }).filter_map(|x| x)
    }

    pub fn bounds(&self) -> [T; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
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
fn sort<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero>(values: &mut [u32], boxes: &mut [T], indices: &mut IndexVec, left: usize, right: usize, node_size: usize) {
    if (left / node_size) >= (right / node_size) { return; }

    let pivot = values[(left + right) >> 1];
    let mut i: isize = (left as isize) - 1;
    let mut j: isize = (right as isize) + 1;

    loop {
        loop {
            i += 1;
            if values[i as usize] >= pivot { break; }
        }
        loop {
            j -= 1;
            if values[j as usize] <= pivot { break; }
        }
        if i >= j { break; }
        swap(values, boxes, indices, i as usize, j as usize);
    }

    sort(values, boxes, indices, left, j as usize, node_size);
    sort(values, boxes, indices, (j as usize) + 1, right, node_size);
}

// swap two values and two corresponding boxes
fn swap<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero>(values: &mut [u32], boxes: &mut [T], indices: &mut IndexVec, i: usize, j: usize) {
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

    a = A; b = B; c = C; d = D;
    A = (a & (a >> 2)) ^ (b & (b >> 2));
    B = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
    C ^= (a & (c >> 2)) ^ (b & (d >> 2));
    D ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

    a = A; b = B; c = C; d = D;
    A = (a & (a >> 4)) ^ (b & (b >> 4));
    B = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
    C ^= (a & (c >> 4)) ^ (b & (d >> 4));
    D ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

    a = A; b = B; c = C; d = D;
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
