use once_cell::sync::Lazy;

use crate::kdbush::*;

static POINTS: Lazy<Vec<[u32; 2]>> = Lazy::new(|| vec![
    [54,1],[97,21],[65,35],[33,54],[95,39],[54,3],[53,54],[84,72],[33,34],[43,15],[52,83],[81,23],[1,61],[38,74],
    [11,91],[24,56],[90,31],[25,57],[46,61],[29,69],[49,60],[4,98],[71,15],[60,25],[38,84],[52,38],[94,51],[13,25],
    [77,73],[88,87],[6,27],[58,22],[53,28],[27,91],[96,98],[93,14],[22,93],[45,94],[18,28],[35,15],[19,81],[20,81],
    [67,53],[43,3],[47,66],[48,34],[46,12],[32,38],[43,12],[39,94],[88,62],[66,14],[84,30],[72,81],[41,92],[26,4],
    [6,76],[47,21],[57,70],[71,82],[50,68],[96,18],[40,31],[78,53],[71,90],[32,14],[55,6],[32,88],[62,32],[21,67],
    [73,81],[44,64],[29,50],[70,5],[6,22],[68,3],[11,23],[20,42],[21,73],[63,86],[9,40],[99,2],[99,76],[56,77],
    [83,6],[21,72],[78,30],[75,53],[41,11],[95,20],[30,38],[96,82],[65,48],[33,18],[87,28],[10,10],[40,34],
    [10,20],[47,29],[46,78]]
);

static IDS: Lazy<Vec<u32>> = Lazy::new(|| vec![
    97,74,95,30,77,38,76,27,80,55,72,90,88,48,43,46,65,39,62,93,9,96,47,8,3,12,15,14,21,41,36,40,69,56,85,78,17,71,44,
    19,18,13,99,24,67,33,37,49,54,57,98,45,23,31,66,68,0,32,5,51,75,73,84,35,81,22,61,89,1,11,86,52,94,16,2,6,25,92,
    42,20,60,58,83,79,64,10,59,53,26,87,4,63,50,7,28,82,70,29,34,91]
);

static COORDS: Lazy<Vec<u32>> = Lazy::new(|| vec![
    10,20,6,22,10,10,6,27,20,42,18,28,11,23,13,25,9,40,26,4,29,50,30,38,41,11,43,12,43,3,46,12,32,14,35,15,40,31,33,18,
    43,15,40,34,32,38,33,34,33,54,1,61,24,56,11,91,4,98,20,81,22,93,19,81,21,67,6,76,21,72,21,73,25,57,44,64,47,66,29,
    69,46,61,38,74,46,78,38,84,32,88,27,91,45,94,39,94,41,92,47,21,47,29,48,34,60,25,58,22,55,6,62,32,54,1,53,28,54,3,
    66,14,68,3,70,5,83,6,93,14,99,2,71,15,96,18,95,20,97,21,81,23,78,30,84,30,87,28,90,31,65,35,53,54,52,38,65,48,67,
    53,49,60,50,68,57,70,56,77,63,86,71,90,52,83,71,82,72,81,94,51,75,53,95,39,78,53,88,62,84,72,77,73,99,76,73,81,88,
    87,96,98,96,82]
);

#[test]
fn creates_an_index() {
    // creates an index
    let mut builder = KDBushBuilder::new_with_node_size(10);
    builder.extend(POINTS.iter());
    let index = builder.finish();

    assert_eq!(index.ids.iter().collect::<Vec<_>>(), IDS.iter().cloned().collect::<Vec<_>>(), "ids are kd-sorted");
    assert_eq!(index.coords, COORDS.iter().cloned().collect::<Vec<_>>(), "coords are kd-sorted");
}

#[test]
fn range_search() {
    // range search
    let mut builder = KDBushBuilder::new_with_node_size(10);
    builder.extend(POINTS.iter());
    let index = builder.finish();

    let result: Vec<_> = index.search_range(20, 30, 50, 70).collect();

    assert_eq!(result, vec![60,20,45,3,17,71,44,19,18,15,69,90,62,96,47,8,77,72], "returns ids");

    for id in result.iter() {
        let p = POINTS[*id];
        if p[0] < 20 || p[0] > 50 || p[1] < 30 || p[1] > 70 {
            panic!("result point in range");
        }
    }
    println!("result points in range");

    for id in IDS.iter() {
        let id = *id as usize;
        let p = POINTS[id];
        if !result.contains(&id) && p[0] >= 20 && p[0] <= 50 && p[1] >= 30 && p[1] <= 70 {
            panic!("outside point not in range");
        }
    }
    println!("outside points not in range");
}

#[test]
fn radius_search() {
    // radius search
    let mut builder = KDBushBuilder::new_with_node_size(10);
    builder.extend(POINTS.iter());
    let index = builder.finish();

    let qp = [50, 50];
    let r = 20;
    let r2 = 20 * 20;

    let result: Vec<_> = index.search_within(qp[0], qp[1], r).collect();

    assert_eq!(result, [60,6,25,92,42,20,45,3,71,44,18,96], "returns ids");

    for id in result.iter() {
        let p = POINTS[*id];
        if sq_dist(p, qp) > r2 {
            panic!("result point in range");
        }
    }
    println!("result points in range");

    for id in IDS.iter() {
        let id = *id as usize;
        let p = POINTS[id];
        if !result.contains(&id) && sq_dist(p, qp) <= r2 {
            panic!("outside point not in range");
        }
    }
    println!("outside points not in range");
}

fn sq_dist(a: [u32; 2], b: [u32; 2]) -> u32 {
    let dx = if a[0] > b[0] { a[0] - b[0] } else { b[0] - a[0] };
    let dy = if a[1] > b[1] { a[1] - b[1] } else { b[1] - a[1] };
    dx * dx + dy * dy
}
