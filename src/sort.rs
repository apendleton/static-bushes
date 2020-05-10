use crate::{ AllowedNumber, IndexVec };

pub fn sort_kd<T: AllowedNumber>(ids: &mut IndexVec, coords: &mut [T], node_size: usize, left: usize, right: usize, axis: usize) {
    if right - left <= node_size { return; }

    let m = (left + right) >> 1; // middle index

    // sort ids and coords around the middle index so that the halves lie
    // either left/right or top/bottom correspondingly (taking turns)
    select(ids, coords, m, left, right, axis);

    // recursively kd-sort first half and second half on the opposite axis
    sort_kd(ids, coords, node_size, left, m - 1, 1 - axis);
    sort_kd(ids, coords, node_size, m + 1, right, 1 - axis);
}

// custom Floyd-Rivest selection algorithm: sort ids and coords so that
// [left..k-1] items are smaller than k-th item (on either x or y axis)
fn select<T: AllowedNumber>(ids: &mut IndexVec, coords: &mut [T], k: usize, mut left: usize, mut right: usize, axis: usize) {

    while right > left {
        if right - left > 600 {
            let n = (right - left + 1) as f64;
            let m = (k - left + 1) as f64;
            let fk = k as f64;

            let z = n.ln();
            let s = 0.5 * (2.0 * z / 3.0).exp();
            let sd = 0.5 * (z * s * (n - s) / n).sqrt() * (if m - n / 2.0 < 0.0 { -1.0 } else { 1.0 });
            let new_left = max(left, (fk - m * s / n + sd).floor() as usize);
            let new_right = min(right, (fk + (n - m) * s / n + sd).floor() as usize);
            select(ids, coords, k, new_left, new_right, axis);
        }

        let t = coords[2 * k + axis];
        let mut i = left;
        let mut j = right;

        swap_item(ids, coords, left, k);
        if coords[2 * right + axis] > t { swap_item(ids, coords, left, right); }

        while i < j {
            swap_item(ids, coords, i, j);
            i += 1;
            j -= 1;
            while coords[2 * i + axis] < t { i+= 1 };
            while coords[2 * j + axis] > t { j-= 1 };
        }

        if coords[2 * left + axis] == t {
            swap_item(ids, coords, left, j)
        } else {
            j += 1;
            swap_item(ids, coords, j, right);
        }

        if j <= k { left = j + 1; }
        if k <= j { right = j - 1; }
    }
}

fn swap_item<T: AllowedNumber>(ids: &mut IndexVec, coords: &mut [T], i: usize, j: usize) {
    ids.swap(i, j);
    coords.swap(2 * i, 2 * j);
    coords.swap(2 * i + 1, 2 * j + 1);
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
