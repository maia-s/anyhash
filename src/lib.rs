//! Hashing algorithms and tools.
//!
//! This crate provides the traits [`Hash`], [`Hasher`] and [`BuildHasher`], which are exactly
//! like their counterparts in `core`/`std`, except that they're generic over the type of the hash.
//! [`Hasher`] provides some extra methods.
//!
//! It also optionally provides some hash algorithms that implement these traits.

#![no_std]
#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", feature(hasher_prefixfree_extras))]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
macro_rules! test_bytes_hash {
    ($([$hashfn:ident] $($bs:ident: $hash:expr),* $(,)?)*) => { $(
        mod $hashfn {
            use super::*;
            $(
                #[test]
                fn $bs() {
                    assert_eq!($hashfn($crate::tests::RawBytes::new(
                        stringify!($bs).as_bytes()
                    )), $hash);
                }
            )*
        }
    )* };
}

#[macro_export]
/// Implement `::core::Hash::Hash` for types that already implement [`Hash<u64>`].
macro_rules! impl_core_hash {
    ($($t:ty),* $(,)?) => { $(
        impl ::core::hash::Hash for $t {
            #[inline(always)]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                struct Wrap<'a, H: ::core::hash::Hasher>(&'a mut H);
                impl<H: ::core::hash::Hasher> $crate::Hasher<u64> for Wrap<'_, H> {
                    #[inline(always)]
                    fn finish(&self) -> u64 {
                        H::finish(self.0)
                    }

                    #[inline(always)]
                    fn write(&mut self, bytes: &[u8]) {
                        H::write(self.0, bytes)
                    }

                    #[cfg(feature = "nightly")]
                    #[inline(always)]
                    fn write_length_prefix(&mut self, len: usize) {
                        H::write_length_prefix(self.0, len)
                    }

                    #[cfg(feature = "nightly")]
                    #[inline(always)]
                    fn write_str(&mut self, s: &str) {
                        H::write_str(self.0, s)
                    }
                }
                <Self as $crate::Hash<u64>>::hash(self, &mut Wrap(state))
            }
        }
    )* };
}

#[macro_export]
/// Implement `::core::Hash::Hasher` for types that already implement [`Hasher<u64>`].
macro_rules! impl_core_hasher {
    ($($t:ty),* $(,)?) => { $(
        impl ::core::hash::Hasher for $t {
            #[inline(always)]
            fn finish(&self) -> u64 {
                <Self as $crate::Hasher<u64>>::finish(self)
            }

            #[inline(always)]
            fn write(&mut self, bytes: &[u8]) {
                <Self as $crate::Hasher<u64>>::write(self, bytes)
            }
        }
    )* };
}

#[macro_export]
/// Implement `::core::Hash::BuildHasher` for types that already implement [`BuildHasher<u64>`].
macro_rules! impl_core_buildhasher {
    ($($t:ty),* $(,)?) => { $(
        impl ::core::hash::BuildHasher for $t {
            type Hasher = <Self as $crate::BuildHasher<u64>>::Hasher;

            fn build_hasher(&self) -> Self::Hasher {
                <Self as $crate::BuildHasher<u64>>::build_hasher(self)
            }
        }
    )* }
}

#[cfg(feature = "fnv1a")]
pub mod fnv1a;

#[cfg(feature = "xxh64")]
pub mod xxh64;

/// A hashable type.
pub trait Hash<T> {
    /// Feeds this value into the given [`Hasher`].
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher<T>;

    /// Feeds a slice of this type into the given [`Hasher`].
    fn hash_slice<H>(data: &[Self], state: &mut H)
    where
        H: Hasher<T>,
        Self: Sized,
    {
        for data in data {
            data.hash(state);
        }
    }
}

macro_rules! make_hasher_writes {
    ($($t:ty { ne:$ne:ident, le:$le:ident, be:$be:ident }),* $(,)?) => { $(
        #[doc = "Writes a single `"]
        #[doc = stringify!($t)]
        #[doc = "` into this hasher in native byte order."]
        fn $ne(&mut self, i: $t) {
            self.write(&i.to_ne_bytes());
        }

        #[doc = "Writes a single `"]
        #[doc = stringify!($t)]
        #[doc = "` into this hasher in little endian byte order."]
        fn $le(&mut self, i: $t) {
            self.write(&i.to_le_bytes());
        }

        #[doc = "Writes a single `"]
        #[doc = stringify!($t)]
        #[doc = "` into this hasher in big endian byte order."]
        fn $be(&mut self, i: $t) {
            self.write(&i.to_be_bytes());
        }
    )* };
}

/// A trait for hashing an arbitrary stream of bytes.
pub trait Hasher<T> {
    /// Returns the hash value for the values written so far.
    fn finish(&self) -> T;

    /// Writes some data into this hasher.
    fn write(&mut self, bytes: &[u8]);

    /// Writes a single `u8` into this hasher.
    fn write_u8(&mut self, i: u8) {
        self.write(&[i]);
    }

    make_hasher_writes! {
        u16 { ne: write_u16, le: write_u16_le, be: write_u16_be },
        u32 { ne: write_u32, le: write_u32_le, be: write_u32_be },
        u64 { ne: write_u64, le: write_u64_le, be: write_u64_be },
        u128 { ne: write_u128, le: write_u128_le, be: write_u128_be },
        usize { ne: write_usize, le: write_usize_le, be: write_usize_be },
    }

    /// Writes a single `i8` into this hasher.
    fn write_i8(&mut self, i: i8) {
        self.write(&[i as u8]);
    }

    make_hasher_writes! {
        i16 { ne: write_i16, le: write_i16_le, be: write_i16_be },
        i32 { ne: write_i32, le: write_i32_le, be: write_i32_be },
        i64 { ne: write_i64, le: write_i64_le, be: write_i64_be },
        i128 { ne: write_i128, le: write_i128_le, be: write_i128_be },
        isize { ne: write_isize, le: write_isize_le, be: write_isize_be },
    }

    #[cfg(feature = "nightly")]
    /// Writes a length prefix into this hasher, as part of being prefix-free.
    ///
    /// Experimental; see <https://github.com/rust-lang/rust/issues/96762>
    fn write_length_prefix(&mut self, len: usize) {
        self.write_usize(len);
    }

    #[cfg(feature = "nightly")]
    /// Writes a single str into this hasher.
    ///
    /// Experimental; see <https://github.com/rust-lang/rust/issues/96762>
    fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
        self.write_u8(0xff);
    }
}

/// A trait for creating instances of [`Hasher`].
pub trait BuildHasher<T> {
    /// Type of the hasher that will be created.
    type Hasher: Hasher<T>;

    /// Creates a new hasher.
    fn build_hasher(&self) -> Self::Hasher;

    /// Calculates the hash of a single value.
    fn hash_one(&self, x: impl Hash<T>) -> T {
        let mut hasher = self.build_hasher();
        x.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{Hash, Hasher};

    impl_core_hash!(Empty, RawBytes<'_>);

    pub struct Empty;

    impl<T> Hash<T> for Empty {
        fn hash<H>(&self, _: &mut H)
        where
            H: super::Hasher<T>,
        {
        }
    }

    pub struct RawBytes<'a>(&'a [u8]);

    impl<'a> RawBytes<'a> {
        pub const fn new(bytes: &'a [u8]) -> Self {
            Self(bytes)
        }
    }

    impl<T> Hash<T> for RawBytes<'_> {
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            state.write(self.0);
        }
    }
}
