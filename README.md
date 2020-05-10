# flatbush-rs

This is library contains Rust ports of two excellent spatial indexing libraries by @mourner: [KDBush](https://github.com/mourner/kdbush) and [Flatbush](https://github.com/mourner/flatbush) (incomplete: lacks nearest-neighbor search).

Where appropriate, function signatures have been modified as compared to their JS versions either to make the two modules more consistent with one another, or to use more-idiomatic Rust (e.g., structures implement `FromIterator` and can be constructed via `.collect()`).
