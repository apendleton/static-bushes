# flatbush-rs

This is an incomplete port of @mourner's excellent [Flatbush](https://github.com/mourner/flatbush) library. It so far supports only searching, and not nearest-neighbor search. Interfaces are mostly the same, with the big difference so far that the `search` method returns an iterator and lazily produces results, rather than eagerly producing an array up front.