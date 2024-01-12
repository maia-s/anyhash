# hash-t

This crate provides the traits [`Hash`], [`Hasher`] and [`BuildHasher`], which are exactly
like their counterparts in `core`/`std`, except that they're generic over the type of the hash.
[`Hasher`] provides some extra methods.

It also optionally provides some hash algorithms that implement these traits.
