use once_cell::sync::Lazy;

use crate::*;

static DATA: Lazy<Vec<u32>> = Lazy::new(|| vec![
    8, 62, 11, 66, 57, 17, 57, 19, 76, 26, 79, 29, 36, 56, 38, 56, 92, 77, 96, 80, 87, 70, 90, 74,
    43, 41, 47, 43, 0, 58, 2, 62, 76, 86, 80, 89, 27, 13, 27, 15, 71, 63, 75, 67, 25, 2, 27, 2, 87,
    6, 88, 6, 22, 90, 23, 93, 22, 89, 22, 93, 57, 11, 61, 13, 61, 55, 63, 56, 17, 85, 21, 87, 33,
    43, 37, 43, 6, 1, 7, 3, 80, 87, 80, 87, 23, 50, 26, 52, 58, 89, 58, 89, 12, 30, 15, 34, 32, 58,
    36, 61, 41, 84, 44, 87, 44, 18, 44, 19, 13, 63, 15, 67, 52, 70, 54, 74, 57, 59, 58, 59, 17, 90,
    20, 92, 48, 53, 52, 56, 92, 68, 92, 72, 26, 52, 30, 52, 56, 23, 57, 26, 88, 48, 88, 48, 66, 13,
    67, 15, 7, 82, 8, 86, 46, 68, 50, 68, 37, 33, 38, 36, 6, 15, 8, 18, 85, 36, 89, 38, 82, 45, 84,
    48, 12, 2, 16, 3, 26, 15, 26, 16, 55, 23, 59, 26, 76, 37, 79, 39, 86, 74, 90, 77, 16, 75, 18,
    78, 44, 18, 45, 21, 52, 67, 54, 71, 59, 78, 62, 78, 24, 5, 24, 8, 64, 80, 64, 83, 66, 55, 70,
    55, 0, 17, 2, 19, 15, 71, 18, 74, 87, 57, 87, 59, 6, 34, 7, 37, 34, 30, 37, 32, 51, 19, 53, 19,
    72, 51, 73, 55, 29, 45, 30, 45, 94, 94, 96, 95, 7, 22, 11, 24, 86, 45, 87, 48, 33, 62, 34, 65,
    18, 10, 21, 14, 64, 66, 67, 67, 64, 25, 65, 28, 27, 4, 31, 6, 84, 4, 85, 5, 48, 80, 50, 81, 1,
    61, 3, 61, 71, 89, 74, 92, 40, 42, 43, 43, 27, 64, 28, 66, 46, 26, 50, 26, 53, 83, 57, 87, 14,
    75, 15, 79, 31, 45, 34, 45, 89, 84, 92, 88, 84, 51, 85, 53, 67, 87, 67, 89, 39, 26, 43, 27, 47,
    61, 47, 63, 23, 49, 25, 53, 12, 3, 14, 5, 16, 50, 19, 53, 63, 80, 64, 84, 22, 63, 22, 64, 26,
    66, 29, 66, 2, 15, 3, 15, 74, 77, 77, 79, 64, 11, 68, 11, 38, 4, 39, 8, 83, 73, 87, 77, 85, 52,
    89, 56, 74, 60, 76, 63, 62, 66, 65, 67
]);

fn create_index() -> Flatbush<u32> {
    let mut builder = FlatbushBuilder::new(DATA.len() / 4, None);

    for i in (0..DATA.len()).step_by(4) {
        builder.add(DATA[i], DATA[i + 1], DATA[i + 2], DATA[i + 3]);
    }
    let index = builder.finish();

    return index;
}

fn create_small_index(num_items: usize, node_size: usize) -> Flatbush<u32> {
    let mut builder = FlatbushBuilder::new(num_items, Some(node_size));
    for i in (0..(4 * num_items)).step_by(4) {
        builder.add(DATA[i], DATA[i + 1], DATA[i + 2], DATA[i + 3]);
    }
    let index = builder.finish();
    return index;
}


#[test]
fn indexes_a_bunch_of_rectangles() {
	// indexes a bunch of rectangles
    let index = create_index();

    let len = index.boxes.len();
    assert_eq!(index.boxes.len() + index.indices.iter().count(), 540);
    assert_eq!(index.boxes[(len - 4)..len], [0, 1, 96, 95]);
    assert_eq!(index.indices.get(len / 4 - 1), 400);

}

#[test]
fn skips_sorting_less_than_node_size_number_of_rectangles() {
	// skips sorting less than node_size number of rectangles
    let num_items = 14;
    let node_size = 16;
    let index = create_small_index(num_items, node_size);

    // compute expected root box extents
    let mut root_x_min = u32::MAX;
    let mut root_y_min = u32::MAX;
    let mut root_x_max = u32::MIN;
    let mut root_y_max = u32::MIN;
    for i in (0..(4 * num_items)).step_by(4) {
        if DATA[i] < root_x_min { root_x_min = DATA[i]; }
        if DATA[i + 1] < root_y_min { root_y_min = DATA[i + 1]; }
        if DATA[i + 2] > root_x_max { root_x_max = DATA[i + 2]; }
        if DATA[i + 3] > root_y_max { root_y_max = DATA[i + 3]; }
    }

    // sort should be skipped, ordered progressing indices expected
    let mut expected_indices: Vec<u32> = (0u32..(num_items as u32)).collect();
    expected_indices.push(0);

    let len = index.boxes.len();

    assert_eq!(index.indices.iter().collect::<Vec<_>>(), expected_indices);
    assert_eq!(len, (num_items + 1) * 4);
    assert_eq!(index.boxes[(len - 4)..len], [root_x_min, root_y_min, root_x_max, root_y_max]);

}

#[test]
fn performs_bbox_search() {
	// performs bbox search
    let index = create_index();

    let ids = index.search(40, 40, 60, 60);

    let mut results = Vec::new();
    for id in ids {
        results.push(DATA[4 * id]);
        results.push(DATA[4 * id + 1]);
        results.push(DATA[4 * id + 2]);
        results.push(DATA[4 * id + 3]);
    }

    assert_eq!(results.sort(), [57, 59, 58, 59, 48, 53, 52, 56, 40, 42, 43, 43, 43, 41, 47, 43].sort());

}

#[test]
fn performs_bbox_search_signed_float() {
	// performs bbox search
    let mut builder = FlatbushBuilder::new(DATA.len() / 4, None);

    let data: Vec<f64> = DATA.iter().map(|d| (*d as f64) - 100.0).collect();

    for i in (0..data.len()).step_by(4) {
        builder.add(data[i], data[i + 1], data[i + 2], data[i + 3]);
    }
    let index = builder.finish();

    let ids = index.search(-60.0, -60.0, -40.0, -40.0);

    let mut results = Vec::new();
    for id in ids {
        results.push(data[4 * id]);
        results.push(data[4 * id + 1]);
        results.push(data[4 * id + 2]);
        results.push(data[4 * id + 3]);
    }

    let mut expected = vec![-43.0, -41.0, -42.0, -41.0, -52.0, -47.0, -48.0, -44.0, -60.0, -58.0, -57.0, -57.0, -57.0, -59.0, -53.0, -57.0];

    results.sort_by(|a, b| a.partial_cmp(b).unwrap());
    expected.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert_eq!(results, expected);
}

#[test]
#[should_panic]
fn throws_an_error_if_added_less_items_than_the_index_size() {
	// throws an error if added less items than the index size
    let builder: FlatbushBuilder<u32> = FlatbushBuilder::new(DATA.len() / 4, None);
    let _index = builder.finish();
}

#[test]
fn returns_index_of_newly_added_rectangle() {
	// returns index of newly-added rectangle
    let count = 5;
    let mut builder = FlatbushBuilder::new(count, None);

    let mut ids = vec![];
    for i in 0..count {
        let id = builder.add(DATA[i], DATA[i + 1], DATA[i + 2], DATA[i + 3]);
        ids.push(id);
    }

    let expected_sequence: Vec<usize> = (0..5).collect();
    assert_eq!(ids, expected_sequence);
}