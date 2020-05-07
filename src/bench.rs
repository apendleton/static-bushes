use crate::*;

use std::time::Instant;

use rand::Rng;
use num_traits::{FromPrimitive, AsPrimitive};

pub const N: usize = 1000000;
pub const K: usize = 1000;
pub const NODE_SIZE: usize = 16;

fn add_random_box<T: FromPrimitive + AsPrimitive<f64>>(arr: &mut Vec<T>, box_size: usize) {
    let mut rng = rand::thread_rng();
    let box_size = box_size as f64;

    let x = T::from_f64(rng.gen::<f64>() * (100.0 - box_size)).unwrap();
    let y = T::from_f64(rng.gen::<f64>() * (100.0 - box_size)).unwrap();
    let x2 = T::from_f64(x.as_() + (rng.gen::<f64>() * box_size)).unwrap();
    let y2 = T::from_f64(y.as_() + (rng.gen::<f64>() * box_size)).unwrap();

    arr.extend(vec![x, y, x2, y2]);
}

fn bench_search<T: PartialOrd + NumOps + AsPrimitive<f64> + Bounded + Zero>(index: &Flatbush<T>, boxes: &[T], name: &str, warmup: bool) {
    let now = Instant::now();
    let id = format!("{} searches {}", K, name);
    for i in (0..boxes.len()).step_by(4) {
        let _results: Vec<_> = index.search(
            boxes[i],
            boxes[i + 1],
            boxes[i + 2],
            boxes[i + 3]
        ).collect();
    }
    let elapsed = now.elapsed().as_secs_f64();
    if !warmup { println!("{} {} {}", std::any::type_name::<T>(), id, elapsed); }
}

fn bench_set<T: PartialOrd + NumOps + AsPrimitive<f64> + FromPrimitive + Bounded + Zero>() {
    let mut coords: Vec<T> = Vec::new();
    for _i in 0..N {
        add_random_box(&mut coords, 1);
    }

    let mut boxes100: Vec<T> = Vec::new();
    let mut boxes10: Vec<T> = Vec::new();
    let mut boxes1: Vec<T> = Vec::new();
    for _i in 0..K {
        add_random_box(&mut boxes100, (100.0 * (0.1_f64).sqrt()) as usize);
        add_random_box(&mut boxes10, 10);
        add_random_box(&mut boxes1, 1);
    }

    let now = Instant::now();
    let mut builder = FlatbushBuilder::new(N, Some(NODE_SIZE));
    for i in (0..coords.len()).step_by(4) {
        builder.add(
            coords[i],
            coords[i + 1],
            coords[i + 2],
            coords[i + 3]
        );
    }
    let index = builder.finish();
    let elapsed = now.elapsed().as_secs_f64();
    println!("flatbush {}", elapsed);

    bench_search(&index, &boxes1, "0.01%", true);
    bench_search(&index, &boxes100, "10%", false);
    bench_search(&index, &boxes10, "1%", false);
    bench_search(&index, &boxes1, "0.01%", false);
}

#[test]
#[ignore]
fn bench() {
    bench_set::<f64>();
    bench_set::<u32>();
}