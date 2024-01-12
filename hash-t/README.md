# `Hash<T>`: A trait for generic hashes

This crate provides the traits [`Hash`], [`Hasher`] and [`BuildHasher`], which are exactly
like their counterparts in `core`/`std`, except that they're generic over the type of the hash.
It also provides tools for working with endian independent hashes.

It also optionally provides some hash algorithms that implement these traits.
