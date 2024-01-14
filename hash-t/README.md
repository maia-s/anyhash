# Generic traits and tools for hashing

This crate provides the traits [`Hash`], [`Hasher`] and [`BuildHasher`], which are exactly
like their counterparts in `core`/`std`, except that they're generic over the type of the hash.
It also provides tools for working with endian independent hashes.

### Features

The crate is `no_std` and uses no optional features by default. The following features are available:

- `alloc`: Enable trait implementations for the standard `alloc` crate.
- `std`: Enable trait implementations for the standard `std` crate. Implies `alloc`.

Optional integrations:

- `bnum`: Implement [`Hash`] for the `bnum` crate's types, and support them as the hash result for built-in hashers that can use them.

Built-in hashers:

- `fnv`: Hashers using the Fnv1 and Fnv1a algorithms.
- `spooky`: Hashers using the SpookyHash algorithm. V1 and V2 are available.
- `xxh64`: Hasher using the Xxh64 algorithm.
