#![no_std]
#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", feature(hasher_prefixfree_extras))]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

extern crate self as hash_t;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::fmt::Debug;
use core::marker::PhantomData;

/// Derive macro for [`Hash<T>`].
pub use hash_t_macros::HashT;

/// Implement `::core::Hash::Hash` for types that already implement [`Hash<u64>`].
///
/// ```
/// # use hash_t::*;
/// # #[derive(HashT)]
/// # struct MyType;
/// // Implements `::core::Hash:Hash` for `MyType`.
/// impl_core_hash!(MyType);
/// ```
///
/// You can pass multiple types as arguments. Types are separated by `;`.
///
/// ```
/// # use hash_t::*;
/// # #[derive(HashT)]
/// # struct MyOtherType<T>(T);
/// // Implements `::core::Hash:Hash` for `MyOtherType<u32>` and `MyOtherType<u64>`.
/// impl_core_hash!(MyOtherType<u32>; MyOtherType<u64>);
/// ```
///
/// You can also pass generic types using the `impl` keyword.
///
/// ```
/// # use hash_t::*;
/// # use core::fmt::Display;
/// # #[derive(HashT)]
/// # struct MyType<T>(T);
/// # #[derive(HashT)]
/// # struct MyOtherType<'a, T, U, V>(core::marker::PhantomData<&'a (T, U, V)>);
/// // Implements `::core::Hash:Hash` for `MyType` and `MyOtherType`.
/// impl_core_hash! {
///     impl<T> MyType<T>;
///     impl<'a, T, U: 'a> MyOtherType<'a, T, u32, U> where Self: Display;
/// }
/// ```
pub use hash_t_macros::impl_core_hash;

/// Implement `::core::Hash::Hasher` for types that already implement [`Hasher<u64>`].
///
/// ```
/// # use hash_t::*;
/// # struct MyType;
/// # impl Hasher<u64> for MyType {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// // Implements `::core::Hash:Hasher` for `MyType`.
/// impl_core_hasher!(MyType);
/// ```
///
/// You can pass multiple types as arguments. Types are separated by `;`.
///
/// ```
/// # use hash_t::*;
/// # struct MyOtherType<T>(T);
/// # impl<T> Hasher<u64> for MyOtherType<T> {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// // Implements `::core::Hash:Hasher` for `MyOtherType<u32>` and `MyOtherType<u64>`.
/// impl_core_hasher!(MyOtherType<u32>; MyOtherType<u64>);
/// ```
///
/// You can also pass generic types using the `impl` keyword.
///
/// ```
/// # use hash_t::*;
/// # use core::fmt::Display;
/// # struct MyType<T>(T);
/// # impl<T> Hasher<u64> for MyType<T> {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// # struct MyOtherType<'a, T, U, V>(core::marker::PhantomData<&'a (T, U, V)>);
/// # impl<'a, T, U, V> Hasher<u64> for MyOtherType<'a, T, U, V> {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// // Implements `::core::Hash:Hasher` for `MyType` and `MyOtherType`.
/// impl_core_hasher! {
///     impl<T> MyType<T>;
///     impl<'a, T, U: 'a> MyOtherType<'a, T, u32, U> where Self: Display;
/// }
/// ```
pub use hash_t_macros::impl_core_hasher;

/// Implement `::core::Hash::BuildHasher` for types that already implement [`BuildHasher<u64>`].
///
/// ```
/// # use hash_t::*;
/// # struct H;
/// # impl Hasher<u64> for H {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// # struct MyType;
/// # impl BuildHasher<u64> for MyType {
/// #   type Hasher = H;
/// #   fn build_hasher(&self) -> Self::Hasher { H }
/// # }
/// // Implements `::core::Hash:BuildHasher` for `MyType`.
/// impl_core_build_hasher!(MyType);
/// ```
///
/// You can pass multiple types as arguments. Types are separated by `;`.
///
/// ```
/// # use hash_t::*;
/// # struct H;
/// # impl Hasher<u64> for H {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// # struct MyOtherType<T>(T);
/// # impl<T> BuildHasher<u64> for MyOtherType<T> {
/// #   type Hasher = H;
/// #   fn build_hasher(&self) -> Self::Hasher { H }
/// # }
/// // Implements `::core::Hash:BuildHasher` for `MyOtherType<u32>` and `MyOtherType<u64>`.
/// impl_core_build_hasher!(MyOtherType<u32>; MyOtherType<u64>);
/// ```
///
/// You can also pass generic types using the `impl` keyword.
///
/// ```
/// # use hash_t::*;
/// # use core::fmt::Display;
/// # struct H;
/// # impl Hasher<u64> for H {
/// #   fn finish(&self) -> u64 { 0 }
/// #   fn write(&mut self, _: &[u8]) {}
/// # }
/// # struct MyType<T>(T);
/// # impl<T> BuildHasher<u64> for MyType<T> {
/// #   type Hasher = H;
/// #   fn build_hasher(&self) -> Self::Hasher { H }
/// # }
/// # struct MyOtherType<'a, T, U, V>(core::marker::PhantomData<&'a (T, U, V)>);
/// # impl<'a, T, U, V> BuildHasher<u64> for MyOtherType<'a, T, U, V> {
/// #   type Hasher = H;
/// #   fn build_hasher(&self) -> Self::Hasher { H }
/// # }
/// // Implements `::core::Hash:BuildHasher` for `MyType` and `MyOtherType`.
/// impl_core_build_hasher! {
///     impl<T> MyType<T>;
///     impl<'a, T, U: 'a> MyOtherType<'a, T, u32, U> where Self: Display;
/// }
/// ```
pub use hash_t_macros::impl_core_build_hasher;

/// Implement [`Hash<T>`] for types that already implement `::core::hash::Hash`.
/// This will panic if `::core::hash::Hasher::finish` is called during hashing.
/// You can use [`impl_hash_u64`] instead to only implement [`Hash<u64>`].
///
/// ```
/// # use hash_t::*;
/// # #[derive(Hash)]
/// # struct MyType;
/// // Implements `Hash<T>` for `MyType`.
/// impl_hash_t!(MyType);
/// ```
///
/// You can pass multiple types as arguments. Types are separated by `;`.
///
/// ```
/// # use hash_t::*;
/// # #[derive(Hash)]
/// # struct MyOtherType<T>(T);
/// // Implements `Hash<T>` for `MyOtherType<u32>` and `MyOtherType<u64>`.
/// impl_hash_t!(MyOtherType<u32>; MyOtherType<u64>);
/// ```
///
/// You can also pass generic types using the `impl` keyword.
///
/// ```
/// # use hash_t::*;
/// # use core::fmt::Display;
/// # #[derive(Hash)]
/// # struct MyType<T>(T);
/// # #[derive(Hash)]
/// # struct MyOtherType<'a, T, U, V>(core::marker::PhantomData<&'a (T, U, V)>);
/// // Implements `Hash<T>` for `MyType` and `MyOtherType`.
/// impl_hash_t! {
///     impl<T> MyType<T>;
///     impl<'a, T, U: 'a> MyOtherType<'a, T, u32, U> where Self: Display;
/// }
/// ```
pub use hash_t_macros::impl_hash_t;

/// Implement [`Hash<u64>`] for types that already implement `::core::hash::Hash`.
/// If you know the hashed type doesn't call `::core::hash::Hasher::finish` during hashing,
/// you can use [`impl_hash_t`] instead to implement [`Hash<T>`] for all `T`
///
/// ```
/// # use hash_t::*;
/// # #[derive(Hash)]
/// # struct MyType;
/// // Implements `Hash<u64>` for `MyType`.
/// impl_hash_u64!(MyType);
/// ```
///
/// You can pass multiple types as arguments. Types are separated by `;`.
///
/// ```
/// # use hash_t::*;
/// # #[derive(Hash)]
/// # struct MyOtherType<T>(T);
/// // Implements `Hash<u64>` for `MyOtherType<u32>` and `MyOtherType<u64>`.
/// impl_hash_u64!(MyOtherType<u32>; MyOtherType<u64>);
/// ```
///
/// You can also pass generic types using the `impl` keyword.
///
/// ```
/// # use hash_t::*;
/// # use core::fmt::Display;
/// # #[derive(Hash)]
/// # struct MyType<T>(T);
/// # #[derive(Hash)]
/// # struct MyOtherType<'a, T, U, V>(core::marker::PhantomData<&'a (T, U, V)>);
/// // Implements `Hash<u64>` for `MyType` and `MyOtherType`.
/// impl_hash_u64! {
///     impl<T> MyType<T>;
///     impl<'a, T, U: 'a> MyOtherType<'a, T, u32, U> where Self: Display;
/// }
/// ```
pub use hash_t_macros::impl_hash_u64;

macro_rules! define_writes_for_hasher {
    (native endian) => {
        define_writes_for_hasher!("hasher.": to_ne_bytes);
    };

    (little endian) => {
        define_writes_for_hasher!("hasher in little endian byte order.": to_le_bytes);
    };

    (big endian) => {
        define_writes_for_hasher!("hasher in big endian byte order.": to_be_bytes);
    };

    ($desc:literal: $c:ident) => {
        /// Writes a single `u8` into this hasher.
        #[inline]
        fn write_u8(&mut self, i: u8) {
            self.write(&[i]);
        }

        define_writes_for_hasher! {
            $desc: $c,
            u16: write_u16,
            u32: write_u32,
            u64: write_u64,
            u128: write_u128,
            usize: write_usize,
        }

        /// Writes a single `i8` into this hasher.
        #[inline]
        fn write_i8(&mut self, i: i8) {
            self.write(&[i as u8]);
        }

        define_writes_for_hasher! {
            $desc: $c,
            i16: write_i16,
            i32: write_i32,
            i64: write_i64,
            i128: write_i128,
            isize: write_isize,
        }

        /// Writes a length prefix into this
        #[doc = $desc]
        #[inline]
        fn write_length_prefix(&mut self, len: usize) {
            self.write_usize(len);
        }

        /// Writes a single str into this
        #[doc = $desc]
        #[inline]
        fn write_str(&mut self, s: &str) {
            self.write(s.as_bytes());
            self.write_u8(0xff);
        }
    };

    ($desc:literal: $c:ident, $($t:ty: $fn:ident),* $(,)*) => {
        $(
            /// Writes a single `
            #[doc = stringify!($t)]
            /// ` into this
            #[doc = $desc]
            #[inline]
            fn $fn(&mut self, i: $t) {
                self.write(&i.$c())
            }
        )*
    };
}

#[cfg(test)]
macro_rules! test_bytes_hash {
    ($([$hashfn:ident] $($bs:ident: $hash:expr),* $(,)?)*) => { $(
        mod $hashfn {
            use super::*;
            $(
                #[test]
                fn $bs() {
                    assert_eq!($hashfn($crate::tests::RawBytes(stringify!($bs).as_bytes())), $hash);
                }
            )*
        }
    )* };
}

macro_rules! impl_hasher_core_fwd_writes {
    ([] $($t:ty: $fn:ident),* $(,)?) => { $(
        #[inline(always)]
        fn $fn(&mut self, i: $t) { H::$fn(self.0, i); }
    )* };
    ([&mut] $($t:ty: $fn:ident),* $(,)?) => { $(
        #[inline(always)]
        fn $fn(&mut self, i: $t) { H::$fn(&mut self.0, i); }
    )* };
}

macro_rules! impl_hasher_core_fwd {
    ($($ref:tt)*) => {
        #[inline(always)]
        fn write(&mut self, bytes: &[u8]) {
            H::write($($ref)* self.0, bytes)
        }

        #[inline(always)]
        fn write_u8(&mut self, i: u8) {
            H::write_u8($($ref)* self.0, i)
        }

        #[inline(always)]
        fn write_i8(&mut self, i: i8) {
            H::write_i8($($ref)* self.0, i)
        }

        impl_hasher_core_fwd_writes! {
            [$($ref)*]
            u16: write_u16,
            u32: write_u32,
            u64: write_u64,
            u128: write_u128,
            usize: write_usize,
            i16: write_i16,
            i32: write_i32,
            i64: write_i64,
            i128: write_i128,
            isize: write_isize
        }

        #[cfg(feature = "nightly")]
        #[inline(always)]
        fn write_length_prefix(&mut self, len: usize) {
            H::write_length_prefix($($ref)* self.0, len)
        }

        #[cfg(feature = "nightly")]
        #[inline(always)]
        fn write_str(&mut self, s: &str) {
            H::write_str($($ref)* self.0, s)
        }
    };
}

#[cfg(feature = "fnv1a")]
pub mod fnv1a;

#[cfg(feature = "spooky")]
pub mod spooky;

#[cfg(feature = "xxh64")]
pub mod xxh64;

mod impls;

/// A hashable type.
pub trait Hash<T> {
    /// Feeds this value into the given [`Hasher`].
    fn hash<H: Hasher<T>>(&self, state: &mut H);

    /// Feeds a slice of this type into the given [`Hasher`].
    fn hash_slice<H: Hasher<T>>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for data in data {
            data.hash(state);
        }
    }
}

/// A trait for hashing an arbitrary stream of bytes.
pub trait Hasher<T> {
    /// Returns the hash value for the values written so far.
    fn finish(&self) -> T;

    /// Writes some data into this hasher.
    fn write(&mut self, bytes: &[u8]);

    define_writes_for_hasher!(native endian);
}

/// A trait for creating instances of [`Hasher`].
pub trait BuildHasher<T> {
    /// Type of the hasher that will be created.
    type Hasher: Hasher<T>;

    /// Creates a new hasher.
    fn build_hasher(&self) -> Self::Hasher;

    /// Calculates the hash of a single value.
    fn hash_one<U: Hash<T>>(&self, x: U) -> T {
        let mut hasher = self.build_hasher();
        x.hash(&mut hasher);
        hasher.finish()
    }
}

/// Marker trait for hashers that, given the same byte stream, calculates the same hash
/// on hosts of different endiannesses.
pub trait EndianIndependentAlgorithm {}

/// Marker trait for hashers whose write methods write data in the same order
/// regardless of the endianness of the host. Be aware that a type may write
/// endian dependent data to the hasher in other ways, so this isn't a guarantee.
///
/// The [`HasherLe`] and [`HasherBe`] types can be used to create hashers that
/// implement this trait.
pub trait EndianIndependentWrites {}

/// Automatically implemented for [`Hasher`]s that implement both [`EndianIndependentAlgorithm`]
/// and [`EndianIndependentWrites`].
pub trait EndianIndependentHasher<T>:
    Hasher<T> + EndianIndependentAlgorithm + EndianIndependentWrites
{
}

impl<T, H> EndianIndependentHasher<T> for H where
    H: ?Sized + Hasher<T> + EndianIndependentAlgorithm + EndianIndependentWrites
{
}

/// Wrapper for types implementing [`Hasher<T>`] to change native endian writes to little endian.
///
/// This can aid in creating an endian independent hash, but be aware that types may write endian
/// dependent data in ways that can't be detected or fixed by this wrapper. The wrapped hasher's
/// algorithm must also be endian independent for this to work.
///
/// Hashers with endian independent algorithms implement the [`EndianIndependentAlgorithm`] trait.
pub struct HasherLe<T, H: Hasher<T>>(H, PhantomData<fn() -> T>);

impl_core_hasher!(impl<T, H: Hasher<T>> HasherLe<T, H>);

impl<T, H: Hasher<T> + EndianIndependentAlgorithm> EndianIndependentAlgorithm for HasherLe<T, H> {}
impl<T, H: Hasher<T>> EndianIndependentWrites for HasherLe<T, H> {}

impl<T, H: Hasher<T>> HasherLe<T, H> {
    /// Create a new `HasherLe`.
    #[inline]
    pub const fn new(hasher: H) -> Self {
        Self(hasher, PhantomData)
    }
}

impl<T, H: Hasher<T>> Hasher<T> for HasherLe<T, H> {
    #[inline]
    fn finish(&self) -> T {
        self.0.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    define_writes_for_hasher!(little endian);
}

impl<T, H: Hasher<T> + Debug> Debug for HasherLe<T, H> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T, H: Hasher<T> + Clone> Clone for HasherLe<T, H> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<T, H: Hasher<T> + Default> Default for HasherLe<T, H> {
    #[inline]
    fn default() -> Self {
        Self::new(H::default())
    }
}

/// Wrapper for types implementing [`Hasher<T>`] to change native endian writes to big endian.
///
/// This can aid in creating an endian independent hash, but be aware that types may write endian
/// dependent data in ways that can't be detected or fixed by this wrapper. The wrapped hasher's
/// algorithm must also be endian independent for this to work.
///
/// Hashers with endian independent algorithms implement the [`EndianIndependentAlgorithm`] trait.
pub struct HasherBe<T, H: Hasher<T>>(H, PhantomData<fn() -> T>);

impl_core_hasher!(impl<T, H: Hasher<T>> HasherBe<T, H>);

impl<T, H: Hasher<T> + EndianIndependentAlgorithm> EndianIndependentAlgorithm for HasherBe<T, H> {}
impl<T, H: Hasher<T>> EndianIndependentWrites for HasherBe<T, H> {}

impl<T, H: Hasher<T>> HasherBe<T, H> {
    /// Create a new `HasherBe`.
    #[inline]
    pub const fn new(hasher: H) -> Self {
        Self(hasher, PhantomData)
    }
}

impl<T, H: Hasher<T>> Hasher<T> for HasherBe<T, H> {
    #[inline]
    fn finish(&self) -> T {
        self.0.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    define_writes_for_hasher!(big endian);
}

impl<T, H: Hasher<T> + Debug> Debug for HasherBe<T, H> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T, H: Hasher<T> + Clone> Clone for HasherBe<T, H> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<T, H: Hasher<T> + Default> Default for HasherBe<T, H> {
    #[inline]
    fn default() -> Self {
        Self::new(H::default())
    }
}

/// `BuildHasher` for making [`HasherLe`] hashers.
pub struct BuildHasherLe<T, BH: BuildHasher<T>>(BH, PhantomData<fn() -> T>);

impl_core_build_hasher!(impl<T, BH: BuildHasher<T>> BuildHasherLe<T, BH>);

impl<T, BH: BuildHasher<T>> BuildHasherLe<T, BH> {
    /// Create a new `BuildHasherLe`.
    #[inline]
    pub const fn new(build_hasher: BH) -> Self {
        Self(build_hasher, PhantomData)
    }
}

impl<T, BH: BuildHasher<T>> BuildHasher<T> for BuildHasherLe<T, BH> {
    type Hasher = HasherLe<T, BH::Hasher>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        HasherLe::<T, BH::Hasher>::new(self.0.build_hasher())
    }
}

impl<T, BH: BuildHasher<T> + Debug> Debug for BuildHasherLe<T, BH> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T, BH: BuildHasher<T> + Clone> Clone for BuildHasherLe<T, BH> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<T, BH: BuildHasher<T> + Default> Default for BuildHasherLe<T, BH> {
    #[inline]
    fn default() -> Self {
        Self::new(BH::default())
    }
}

/// `BuildHasher` for making [`HasherBe`] hashers.
pub struct BuildHasherBe<T, BH: BuildHasher<T>>(BH, PhantomData<fn() -> T>);

impl_core_build_hasher!(impl<T, BH: BuildHasher<T>> BuildHasherBe<T, BH>);

impl<T, BH: BuildHasher<T>> BuildHasherBe<T, BH> {
    /// Create a new `BuildHasherBe`.
    #[inline]
    pub const fn new(build_hasher: BH) -> Self {
        Self(build_hasher, PhantomData)
    }
}

impl<T, BH: BuildHasher<T>> BuildHasher<T> for BuildHasherBe<T, BH> {
    type Hasher = HasherBe<T, BH::Hasher>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        HasherBe::<T, BH::Hasher>::new(self.0.build_hasher())
    }
}

impl<T, BH: BuildHasher<T> + Debug> Debug for BuildHasherBe<T, BH> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T, BH: BuildHasher<T> + Clone> Clone for BuildHasherBe<T, BH> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<T, BH: BuildHasher<T> + Default> Default for BuildHasherBe<T, BH> {
    #[inline]
    fn default() -> Self {
        Self::new(BH::default())
    }
}

#[doc(hidden)]
pub mod internal {
    use core::marker::PhantomData;

    use bytemuck::{cast_mut, cast_ref, Pod};

    use super::*;

    #[repr(transparent)]
    pub struct WrapCoreForT<'a, T, H: Hasher<T>>(&'a mut H, PhantomData<fn() -> T>);

    impl<'a, T, H: Hasher<T>> WrapCoreForT<'a, T, H> {
        #[inline(always)]
        pub fn new(hasher: &'a mut H) -> Self {
            Self(hasher, PhantomData)
        }
    }

    impl<T, H: Hasher<T>> core::hash::Hasher for WrapCoreForT<'_, T, H> {
        #[inline(always)]
        fn finish(&self) -> u64 {
            panic!("`core::hash::Hasher::finish` called while calculating generic hash");
        }

        impl_hasher_core_fwd!();
    }

    #[repr(transparent)]
    pub struct WrapCoreForU64<'a, H: core::hash::Hasher>(&'a mut H);

    impl<'a, H: core::hash::Hasher> WrapCoreForU64<'a, H> {
        #[inline(always)]
        pub fn new(hasher: &'a mut H) -> Self {
            Self(hasher)
        }
    }

    impl<H: ::core::hash::Hasher> Hasher<u64> for WrapCoreForU64<'_, H> {
        #[inline(always)]
        fn finish(&self) -> u64 {
            H::finish(self.0)
        }

        impl_hasher_core_fwd!();
    }

    #[repr(transparent)]
    pub struct WrapU64ForCore<H: Hasher<u64>>(H);

    impl<H: Hasher<u64>> WrapU64ForCore<H> {
        #[inline(always)]
        pub const fn new(hasher: H) -> Self {
            Self(hasher)
        }
    }

    impl<H: Hasher<u64>> ::core::hash::Hasher for WrapU64ForCore<H> {
        #[inline(always)]
        fn finish(&self) -> u64 {
            H::finish(&self.0)
        }

        impl_hasher_core_fwd!(&mut);
    }

    pub(crate) trait ConstValue: Copy + Default {
        const VALUE: usize;
        type ArrayU64: Pod + Default;
        type Array2xU32: Pod;
        type Array4xU16: Pod;
        type Array8xU8: Pod;
    }

    macro_rules! define_const_values {
        ($($name:ident = $value:expr),* $(,)?) => { $(
            #[derive(Clone, Copy, Default)]
            pub(crate) struct $name;

            impl ConstValue for $name {
                const VALUE: usize = $value;
                type ArrayU64 = [u64; $value];
                type Array2xU32 = [u32; 2 * $value];
                type Array4xU16 = [u16; 4 * $value];
                type Array8xU8 = [u8; 8 * $value];
            }

            impl From<$name> for usize {
                #[inline]
                fn from(_: $name) -> Self {
                    $name::VALUE
                }
            }
        )* };
    }

    define_const_values! {
        N4 = 4,
        N24 = 24,
    }

    #[cfg(feature = "bytemuck")]
    #[derive(Clone, Copy, Default)]
    #[repr(transparent)]
    pub(crate) struct Buffer<N: ConstValue>(N::ArrayU64);

    #[cfg(feature = "bytemuck")]
    #[allow(dead_code)]
    impl<N: ConstValue> Buffer<N> {
        #[inline]
        pub fn new() -> Self {
            Self(N::ArrayU64::default())
        }

        #[inline]
        pub fn as_bytes(&self) -> &N::Array8xU8 {
            cast_ref(&self.0)
        }

        #[inline]
        pub fn as_bytes_mut(&mut self) -> &mut N::Array8xU8 {
            cast_mut(&mut self.0)
        }

        #[inline]
        pub fn as_u16s(&self) -> &N::Array4xU16 {
            cast_ref(&self.0)
        }

        #[inline]
        pub fn as_u16s_mut(&mut self) -> &mut N::Array4xU16 {
            cast_mut(&mut self.0)
        }

        #[inline]
        pub fn as_u32s(&self) -> &N::Array2xU32 {
            cast_ref(&self.0)
        }

        #[inline]
        pub fn as_u32s_mut(&mut self) -> &mut N::Array2xU32 {
            cast_mut(&mut self.0)
        }

        #[inline]
        pub fn as_u64s(&self) -> &N::ArrayU64 {
            &self.0
        }

        #[inline]
        pub fn as_u64s_mut(&mut self) -> &mut N::ArrayU64 {
            &mut self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct RawBytes<'a>(pub &'a [u8]);

    impl<T> Hash<T> for RawBytes<'_> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            Hash::<T>::hash_slice(self.0, state)
        }
    }
}
