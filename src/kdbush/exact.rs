use crate::{ KDBush, AllowedNumber };

use genawaiter::rc::Gen;

impl<T: AllowedNumber> KDBush<T> {
    pub fn exact<'a>(&'a self, qx: T, qy: T) -> impl Iterator<Item = usize> + 'a {
        let mut stack = vec![0, self.ids.len() - 1, 0];

        Gen::new(|co| async move {
            // recursively search for items in range in the kd-sorted arrays
            while stack.len() > 0 {
                // we always push three at a time, so pops three at a time will always work -- unwrap
                // is safe here
                let axis = stack.pop().unwrap();
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                // if we reached "tree node", search linearly
                if right - left <= self.node_size {
                    for i in left..=right {
                        if self.coords[2 * i] == qx && self.coords[2 * i + 1] == qy { co.yield_(self.ids.get(i) as usize).await; }
                    }
                    continue;
                }

                // otherwise find the middle index
                let m = (left + right) >> 1;

                // include the middle item if it's in range
                let x = self.coords[2 * m];
                let y = self.coords[2 * m + 1];
                if x == qx && y == qy { co.yield_(self.ids.get(m) as usize).await; }

                // queue search in halves that intersect the query
                let (over_min, under_max) = if axis == 0 {
                    (qx <= x, qx >= x)
                } else {
                    (qy <= y, qy >= y)
                };

                if over_min {
                    stack.push(left);
                    stack.push(m - 1);
                    stack.push(1 - axis);
                }
                if under_max {
                    stack.push(m + 1);
                    stack.push(right);
                    stack.push(1 - axis);
                }
            }
        }).into_iter()
    }
}
