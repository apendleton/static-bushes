use crate::{ KDBush, AllowedNumber };

use genawaiter::rc::Gen;

impl<T: AllowedNumber> KDBush<T> {
    pub fn within<'a>(&'a self, qx: T, qy: T, r: T) -> impl Iterator<Item = usize> + 'a {
        let mut stack = vec![0, self.ids.len() - 1, 0];
        let r2 = r * r;

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
                        if sq_dist(self.coords[2 * i], self.coords[2 * i + 1], qx, qy) <= r2 { co.yield_(self.ids.get(i) as usize).await; }
                    }
                    continue;
                }

                // otherwise find the middle index
                let m = (left + right) >> 1;

                // include the middle item if it's in range
                let x = self.coords[2 * m];
                let y = self.coords[2 * m + 1];
                if sq_dist(x, y, qx, qy) <= r2 { co.yield_(self.ids.get(m) as usize).await; }

                // queue search in halves that intersect the query
                let (over_min, under_max) = if axis == 0 {
                    (qx - r <= x, qx + r >= x)
                } else {
                    (qy - r <= y, qy + r >= y)
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

fn sq_dist<T: AllowedNumber>(ax: T, ay: T, bx: T, by: T) -> T {
    // T might be unsigned, so we need to jump through some hoops to keep from overflowing
    // (in the future it might make sense to specialize here, and do a faster one for signed ints)
    let dx = if ax > bx { ax - bx } else { bx - ax };
    let dy = if ay > by { ay - by } else { by - ay };
    dx * dx + dy * dy
}
