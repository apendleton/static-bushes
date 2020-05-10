use crate::kdbush::*;

use std::time::Instant;
use std::any::{Any, type_name};

use rand::Rng;
use num_traits::{FromPrimitive, AsPrimitive, Zero};
use flatbush_rs::{FlatbushBuilder, Flatbush};

fn random_val<T: FromPrimitive + AsPrimitive<f64>>(max: f64) -> T {
    let mut rng = rand::thread_rng();
    T::from_f64(rng.gen::<f64>() * max).unwrap()
}

fn random_point<T: FromPrimitive + AsPrimitive<f64>>(max: f64) -> [T; 2] {
    [random_val(max), random_val(max)]
}

use once_cell::sync::Lazy;

static POINTS_U32: Lazy<Vec<[u32; 2]>> = Lazy::new(|| (0..1000000).map(|_| random_point(1000.0)).collect());
static POINTS_F64: Lazy<Vec<[f64; 2]>> = Lazy::new(|| (0..1000000).map(|_| random_point(1000.0)).collect());

static KDBUSH_U32: Lazy<KDBush<u32>> = Lazy::new(|| POINTS_U32.iter().collect());
static KDBUSH_F64: Lazy<KDBush<f64>> = Lazy::new(|| POINTS_F64.iter().collect());

static FLATBUSH_U32: Lazy<Flatbush<u32>> = Lazy::new(|| {
    let mut builder = FlatbushBuilder::new(POINTS_U32.len(), None);
    for coords in POINTS_U32.iter() { builder.add(coords[0], coords[1], coords[0], coords[1]); }
    builder.finish()
});
static FLATBUSH_F64: Lazy<Flatbush<f64>> = Lazy::new(|| {
    let mut builder = FlatbushBuilder::new(POINTS_U32.len(), None);
    for coords in POINTS_F64.iter() { builder.add(coords[0], coords[1], coords[0], coords[1]); }
    builder.finish()
});

fn time<F>(name: &str, f: F) where F: Fn() -> () {
    let now = Instant::now();
    f();
    let elapsed = now.elapsed().as_secs_f64();
    println!("{}: {}", name, elapsed);
}

#[test]
#[ignore]
fn build_kdbush_u32() {
    time("build u32 kdbush", || { Lazy::force(&KDBUSH_U32); });
}

#[test]
#[ignore]
fn build_kdbush_f64() {
    time("build f64 kdbush", || { Lazy::force(&KDBUSH_F64); });
}

#[test]
#[ignore]
fn build_flatbush_u32() {
    time("build u32 flatbush", || { Lazy::force(&FLATBUSH_U32); });
}

#[test]
#[ignore]
fn build_flatbush_f64() {
    time("build f64 flatbush", || { Lazy::force(&FLATBUSH_F64); });
}

fn get_kdbush<T: AllowedNumber + Any>() -> &'static KDBush<T> {
    let kdbush_u32 = &*KDBUSH_U32 as &dyn Any;
    let kdbush_f64 = &*KDBUSH_F64 as &dyn Any;

    if let Some(bush) = kdbush_u32.downcast_ref::<KDBush<T>>() {
        bush
    } else if let Some(bush) = kdbush_f64.downcast_ref::<KDBush<T>>() {
        bush
    } else {
        panic!("must be u32 or f64")
    }
}

fn get_flatbush<T: AllowedNumber + Any + Zero + AsPrimitive<f64>>() -> &'static Flatbush<T> {
    let flatbush_u32 = &*FLATBUSH_U32 as &dyn Any;
    let flatbush_f64 = &*FLATBUSH_F64 as &dyn Any;

    if let Some(bush) = flatbush_u32.downcast_ref::<Flatbush<T>>() {
        bush
    } else if let Some(bush) = flatbush_f64.downcast_ref::<Flatbush<T>>() {
        bush
    } else {
        panic!("must be u32 or f64")
    }
}

fn many_small_bbox_queries<T: AllowedNumber + Any + FromPrimitive + AsPrimitive<f64>>() {
    let index: &KDBush<T> = get_kdbush();
    let one = T::from_u32(1).unwrap();
    time(&format!("10000 small bbox queries {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.range(p[0] - one, p[1] - one, p[0] + one, p[1] + one).collect();
        }
    });
}

#[test]
#[ignore]
fn many_small_bbox_queries_f64() {
    many_small_bbox_queries::<f64>();
}

#[test]
#[ignore]
fn many_small_bbox_queries_u32() {
    many_small_bbox_queries::<u32>();
}

fn many_small_bbox_queries_flatbush<T: AllowedNumber + Any + Zero + FromPrimitive + AsPrimitive<f64>>() {
    let index: &Flatbush<T> = get_flatbush();
    let one = T::from_u32(1).unwrap();
    time(&format!("10000 small bbox queries (flatbush) {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.search(p[0] - one, p[1] - one, p[0] + one, p[1] + one).collect();
        }
    });
}

#[test]
#[ignore]
fn many_small_bbox_queries_flatbush_f64() {
    many_small_bbox_queries_flatbush::<f64>();
}

#[test]
#[ignore]
fn many_small_bbox_queries_flatbush_u32() {
    many_small_bbox_queries_flatbush::<u32>();
}

fn many_small_radius_queries<T: AllowedNumber + Any + FromPrimitive + AsPrimitive<f64>>() {
    let index: &KDBush<T> = get_kdbush();
    let one = T::from_u32(1).unwrap();
    time(&format!("10000 small radius queries {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.within(p[0], p[1], one).collect();
        }
    });
}

#[test]
#[ignore]
fn many_small_radius_queries_f64() {
    many_small_radius_queries::<f64>();
}

#[test]
#[ignore]
fn many_small_radius_queries_u32() {
    many_small_radius_queries::<u32>();
}

fn many_exact_queries<T: AllowedNumber + Any + FromPrimitive + AsPrimitive<f64>>() {
    let index: &KDBush<T> = get_kdbush();
    time(&format!("10000 exact queries {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.range(p[0], p[1], p[0], p[1]).collect();
        }
    });
}

#[test]
#[ignore]
fn many_exact_queries_f64() {
    many_exact_queries::<f64>();
}

#[test]
#[ignore]
fn many_exact_queries_u32() {
    many_exact_queries::<u32>();
}

fn many_exact_queries_radius<T: AllowedNumber + Any + FromPrimitive + AsPrimitive<f64>>() {
    let index: &KDBush<T> = get_kdbush();
    let zero = T::from_u32(0).unwrap();
    time(&format!("10000 exact queries (radius) {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.within(p[0], p[1], zero).collect();
        }
    });
}

#[test]
#[ignore]
fn many_exact_queries_radius_f64() {
    many_exact_queries_radius::<f64>();
}

#[test]
#[ignore]
fn many_exact_queries_radius_u32() {
    many_exact_queries_radius::<u32>();
}

fn many_exact_queries_exact<T: AllowedNumber + Any + FromPrimitive + AsPrimitive<f64>>() {
    let index: &KDBush<T> = get_kdbush();
    time(&format!("10000 exact queries (exact) {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.exact(p[0], p[1]).collect();
        }
    });
}

#[test]
#[ignore]
fn many_exact_queries_exact_f64() {
    many_exact_queries_exact::<f64>();
}

#[test]
#[ignore]
fn many_exact_queries_exact_u32() {
    many_exact_queries_exact::<u32>();
}

fn many_exact_queries_flatbush<T: AllowedNumber + Any + Zero + FromPrimitive + AsPrimitive<f64>>() {
    let index: &Flatbush<T> = get_flatbush();
    time(&format!("10000 exact queries (flatbush) {}", type_name::<T>()), || {
        for _i in 0..10000 {
            let p: [T; 2] = random_point(1000.0);
            let _result: Vec<_> = index.search(p[0], p[1], p[0], p[1]).collect();
        }
    });
}

#[test]
#[ignore]
fn many_exact_queries_flatbush_f64() {
    many_exact_queries_flatbush::<f64>();
}

#[test]
#[ignore]
fn many_exact_queries_flatbush_u32() {
    many_exact_queries_flatbush::<u32>();
}
