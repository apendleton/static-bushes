#![allow(non_snake_case)]

use static_bushes::flatbush::FlatBush;
use static_bushes::kdbush::*;

use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::{AsPrimitive, FromPrimitive};
use rand::Rng;

fn random_val<T: FromPrimitive + AsPrimitive<f64>>(max: f64) -> T {
    let mut rng = rand::thread_rng();
    T::from_f64(rng.gen::<f64>() * max).unwrap()
}

fn random_point<T: FromPrimitive + AsPrimitive<f64>>(max: f64) -> [T; 2] {
    [random_val(max), random_val(max)]
}

fn point_benches(c: &mut Criterion) {
    let INDEX_POINTS_U32: Vec<[u32; 2]> = (0..1000000).map(|_| random_point(1000.0)).collect();
    let INDEX_POINTS_F64: Vec<[f64; 2]> = (0..1000000).map(|_| random_point(1000.0)).collect();

    c.bench_function("build_kdbush_u32", |b| {
        b.iter(|| {
            let _bush: KDBush<_> = INDEX_POINTS_U32.iter().collect();
        })
    });

    c.bench_function("build_kdbush_f64", |b| {
        b.iter(|| {
            let _bush: KDBush<_> = INDEX_POINTS_F64.iter().collect();
        })
    });

    c.bench_function("build_flatbush_u32", |b| {
        b.iter(|| {
            let _bush: FlatBush<_> = INDEX_POINTS_U32
                .iter()
                .map(|coords| [coords[0], coords[1], coords[0], coords[1]])
                .collect();
        })
    });

    c.bench_function("build_flatbush_f64", |b| {
        b.iter(|| {
            let _bush: FlatBush<_> = INDEX_POINTS_F64
                .iter()
                .map(|coords| [coords[0], coords[1], coords[0], coords[1]])
                .collect();
        })
    });

    let KDBUSH_U32: KDBush<u32> = INDEX_POINTS_U32.iter().collect();
    let KDBUSH_F64: KDBush<f64> = INDEX_POINTS_F64.iter().collect();

    let FLATBUSH_U32: FlatBush<u32> = INDEX_POINTS_U32
        .iter()
        .map(|coords| [coords[0], coords[1], coords[0], coords[1]])
        .collect();
    let FLATBUSH_F64: FlatBush<f64> = INDEX_POINTS_F64
        .iter()
        .map(|coords| [coords[0], coords[1], coords[0], coords[1]])
        .collect();

    let SEARCH_POINTS_U32: Vec<[u32; 2]> = (0..1000).map(|_| random_point(1000.0)).collect();
    let SEARCH_POINTS_F64: Vec<[f64; 2]> = (0..1000).map(|_| random_point(1000.0)).collect();

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("exact_bbox_kdbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_U32.search_range(p[0], p[1], p[0], p[1]).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("exact_bbox_kdbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_F64.search_range(p[0], p[1], p[0], p[1]).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("exact_radius_kdbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_U32.search_within(p[0], p[1], 0).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("exact_radius_kdbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_F64.search_within(p[0], p[1], 0.0).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("exact_exact_kdbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_U32.exact(p[0], p[1]).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("exact_exact_kdbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_F64.exact(p[0], p[1]).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("exact_exact_vec_kdbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_U32.exact_as_vec(p[0], p[1]);
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("exact_exact_vec_kdbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_F64.exact_as_vec(p[0], p[1]);
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("exact_bbox_flatbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_U32.search_range(p[0], p[1], p[0], p[1]).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("exact_bbox_flatbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_F64.search_range(p[0], p[1], p[0], p[1]).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("small_bbox_kdbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_U32
                .search_range(p[0].saturating_sub(1), p[1].saturating_sub(1), p[0] + 1, p[1] + 1)
                .collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("small_bbox_kdbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> =
                KDBUSH_F64.search_range(p[0] - 1.0, p[1] - 1.0, p[0] + 1.0, p[1] + 1.0).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("small_bbox_flatbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_U32
                .search_range(p[0].saturating_sub(1), p[1].saturating_sub(1), p[0] + 1, p[1] + 1)
                .collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("small_bbox_flatbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> =
                FLATBUSH_F64.search_range(p[0] - 1.0, p[1] - 1.0, p[0] + 1.0, p[1] + 1.0).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_U32.iter().cycle();
    c.bench_function("small_radius_kdbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_U32.search_within(p[0], p[1], 1).collect();
        })
    });

    let mut cycle = SEARCH_POINTS_F64.iter().cycle();
    c.bench_function("small_radius_kdbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = KDBUSH_F64.search_within(p[0], p[1], 1.0).collect();
        })
    });
}

fn random_box<T: FromPrimitive + AsPrimitive<f64>>(box_size: usize) -> [T; 4] {
    let mut rng = rand::thread_rng();
    let box_size = box_size as f64;

    let x = T::from_f64(rng.gen::<f64>() * (100.0 - box_size)).unwrap();
    let y = T::from_f64(rng.gen::<f64>() * (100.0 - box_size)).unwrap();

    [
        x,
        y,
        T::from_f64(x.as_() + (rng.gen::<f64>() * box_size)).unwrap(),
        T::from_f64(y.as_() + (rng.gen::<f64>() * box_size)).unwrap(),
    ]
}

fn box_benches(c: &mut Criterion) {
    let INDEX_BOXES_U32: Vec<[u32; 4]> = (0..1000000).map(|_| random_box(1)).collect();
    let INDEX_BOXES_F64: Vec<[f64; 4]> = (0..1000000).map(|_| random_box(1)).collect();

    c.bench_function("box_build_flatbush_u32", |b| {
        b.iter(|| {
            let _bush: FlatBush<_> = INDEX_BOXES_U32.iter().collect();
        })
    });

    c.bench_function("box_build_flatbush_f64", |b| {
        b.iter(|| {
            let _bush: FlatBush<_> = INDEX_BOXES_F64.iter().collect();
        })
    });

    let FLATBUSH_U32: FlatBush<u32> = INDEX_BOXES_U32.iter().collect();
    let FLATBUSH_F64: FlatBush<f64> = INDEX_BOXES_F64.iter().collect();

    let SEARCH_BOXES_100_U32: Vec<[u32; 4]> =
        (0..1000).map(|_| random_box((100.0 * (0.1_f64).sqrt()) as usize)).collect();
    let SEARCH_BOXES_10_U32: Vec<[u32; 4]> = (0..1000).map(|_| random_box(10)).collect();
    let SEARCH_BOXES_1_U32: Vec<[u32; 4]> = (0..1000).map(|_| random_box(1)).collect();
    let SEARCH_BOXES_100_F64: Vec<[f64; 4]> =
        (0..1000).map(|_| random_box((100.0 * (0.1_f64).sqrt()) as usize)).collect();
    let SEARCH_BOXES_10_F64: Vec<[f64; 4]> = (0..1000).map(|_| random_box(10)).collect();
    let SEARCH_BOXES_1_F64: Vec<[f64; 4]> = (0..1000).map(|_| random_box(1)).collect();

    let mut cycle = SEARCH_BOXES_100_U32.iter().cycle();
    c.bench_function("box_bbox_100_flatbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_U32.search_range(p[0], p[1], p[2], p[3]).collect();
        })
    });

    let mut cycle = SEARCH_BOXES_100_F64.iter().cycle();
    c.bench_function("box_bbox_100_flatbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_F64.search_range(p[0], p[1], p[2], p[3]).collect();
        })
    });

    let mut cycle = SEARCH_BOXES_10_U32.iter().cycle();
    c.bench_function("box_bbox_10_flatbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_U32.search_range(p[0], p[1], p[2], p[3]).collect();
        })
    });

    let mut cycle = SEARCH_BOXES_10_F64.iter().cycle();
    c.bench_function("box_bbox_10_flatbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_F64.search_range(p[0], p[1], p[2], p[3]).collect();
        })
    });

    let mut cycle = SEARCH_BOXES_1_U32.iter().cycle();
    c.bench_function("box_bbox_1_flatbush_u32", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_U32.search_range(p[0], p[1], p[2], p[3]).collect();
        })
    });

    let mut cycle = SEARCH_BOXES_1_F64.iter().cycle();
    c.bench_function("box_bbox_1_flatbush_f64", |b| {
        b.iter(|| {
            let p = cycle.next().unwrap();
            let _result: Vec<_> = FLATBUSH_F64.search_range(p[0], p[1], p[2], p[3]).collect();
        })
    });
}

criterion_group!(benches, point_benches, box_benches);
criterion_main!(benches);
