# Generic traits and tools for hashing

This crate provides the [`Hash`], [`Hasher`] and [`BuildHasher`] traits, which are almost
exactly like their counterparts in `core`/`std`, except that [`Hasher`] and [`BuildHasher`]
are generic over the type of the hash, and [`Hash`] can be used with any type of hash.
A derive macro for [`Hash`] is available.

The [`Hash`] trait uses the [`HasherWrite`] trait for hashing, which has all the write
methods you're familiar with from `Hasher` in core except for the `finish` method,
because [`Hash`] doesn't know the type of the hash.

Hashing algorithms implement the [`Hasher`] trait. This is generic over the type of the
hash and only contains the `finish` method. It depends on [`HasherWrite`], so you can
use it just like `Hasher` from core.

This crate also provides tools for working with endian independent hashes, and a few
hasher implementations.

### Features

The crate is `no_std` and doesn't enable any features by default. The following features are available:

- `alloc`: Enable trait implementations for the standard `alloc` crate.
- `std`: Enable trait implementations for the standard `std` crate. Implies `alloc`.

Optional integrations:

- `bnum`: Implement [`Hash`] for the `bnum` crate's types, and add support for using them as the hash type for the built-in hashers that can use them.

Built-in hashers:

- `fnv`: Hashers using the Fnv1 and Fnv1a algorithms.
- `spooky`: Hashers using the SpookyHash algorithm. V1 and V2 are available.
- `xxh64`: Hasher using the Xxh64 algorithm.
