//! Hashing traits, algorithms and tools.
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

#[cfg(feature = "alloc")]
extern crate alloc;

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
                    assert_eq!($hashfn($crate::tests::RawBytes(stringify!($bs).as_bytes())), $hash);
                }
            )*
        }
    )* };
}

macro_rules! impl_hasher_deref_writes {
    ($($t:ty { ne:$ne:ident, le:$le:ident, be:$be:ident }),* $(,)?) => { $(
        #[inline]
        fn $ne(&mut self, i: $t) {
            (**self).$ne(i);
        }

        #[inline]
        fn $le(&mut self, i: $t) {
            (**self).$le(i);
        }

        #[inline]
        fn $be(&mut self, i: $t) {
            (**self).$be(i);
        }
    )* };
}

macro_rules! impl_hasher_deref {
    () => {
        #[inline]
        fn finish(&self) -> T {
            (**self).finish()
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            (**self).write(bytes)
        }

        #[inline]
        fn write_u8(&mut self, i: u8) {
            (**self).write_u8(i)
        }

        #[inline]
        fn write_i8(&mut self, i: i8) {
            (**self).write_i8(i)
        }

        impl_hasher_deref_writes! {
            u16 { ne: write_u16, le: write_u16_le, be: write_u16_be },
            u32 { ne: write_u32, le: write_u32_le, be: write_u32_be },
            u64 { ne: write_u64, le: write_u64_le, be: write_u64_be },
            u128 { ne: write_u128, le: write_u128_le, be: write_u128_be },
            usize { ne: write_usize, le: write_usize_le, be: write_usize_be },

            i16 { ne: write_i16, le: write_i16_le, be: write_i16_be },
            i32 { ne: write_i32, le: write_i32_le, be: write_i32_be },
            i64 { ne: write_i64, le: write_i64_le, be: write_i64_be },
            i128 { ne: write_i128, le: write_i128_le, be: write_i128_be },
            isize { ne: write_isize, le: write_isize_le, be: write_isize_be },
        }

        #[inline]
        fn write_length_prefix(&mut self, len: usize) {
            (**self).write_length_prefix(len)
        }

        #[inline]
        fn write_str(&mut self, s: &str) {
            (**self).write_str(s)
        }
    };
}

macro_rules! impl_hasher_fwd_writes {
    ($($t:ty: $ne:ident),* $(,)?) => { $(
        #[inline(always)]
        fn $ne(&mut self, i: $t) {
            H::$ne(self.0, i);
        }
    )* };
}

macro_rules! impl_hasher_fwd {
    () => {
        #[inline(always)]
        fn write(&mut self, bytes: &[u8]) {
            H::write(self.0, bytes)
        }

        #[inline(always)]
        fn write_u8(&mut self, i: u8) {
            H::write_u8(self.0, i)
        }

        #[inline(always)]
        fn write_i8(&mut self, i: i8) {
            H::write_i8(self.0, i)
        }

        impl_hasher_fwd_writes! {
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
            H::write_length_prefix(self.0, len)
        }

        #[cfg(feature = "nightly")]
        #[inline(always)]
        fn write_str(&mut self, s: &str) {
            H::write_str(self.0, s)
        }
    };
}

macro_rules! impl_empty_hash {
    ($($t:ty),* $(,)?) => { $(
        impl<T> Hash<T> for $t {
            fn hash<H: Hasher<T>>(&self, _: &mut H) {}
        }
    )* };
}

macro_rules! impl_hash_from_method {
    ([$T:ident] $($m:ident { $($t:ident $(<
        $($gen:ident $(: $con0:path $(: $con:path)*)?),+
    >)?),* $(,)? })*) => { $($(
        impl<$T $($(, $gen $(: $con0 $(+ $con)*)?)*)?> Hash<$T> for $t $(<$($gen),*>)? {
            #[inline]
            fn hash<H: Hasher<$T>>(&self, state: &mut H) {
                Hash::<$T>::hash(&self.$m(), state)
            }
        }
    )*)* };
}

macro_rules! impl_hash_from_field {
    ([$T:ident] $($n:tt { $($t:ident $(<
        $($gen:ident $(: $con0:path $(: $con:path)*)?),+
    >)?),* $(,)? })*) => { $($(
        impl<$T $($(, $gen $(: $con0 $(+ $con)*)?)*)?> Hash<$T> for $t $(<$($gen),*>)? {
            #[inline]
            fn hash<H: Hasher<$T>>(&self, state: &mut H) {
                Hash::<$T>::hash(&self.$n, state)
            }
        }
    )*)* };
}

#[macro_export]
/// Implement [`Hash<u64>`] for types that already implement `::core::hash::Hash`.
/// If you know the hashed type doesn't call `::core::hash::Hasher::finish`,
/// you can use [`impl_hash_gen`] instead to implement [`Hash<T>`] for all `T`
macro_rules! impl_hash_u64 {
    ($($t:ident $(<
        $($lt0:lifetime $(,$lt:lifetime)*)? $(,)? $($gen:ident $(: $con0:path $(: $con:path)*)?),*
    >)?),* $(,)?) => { $(
        impl<$($($lt0, $($lt,)*)? $($gen: $($con0 $(+ $con)*)?),*)?>
        $crate::Hash<u64> for $t $(<$($lt0, $($lt,)*)? $($gen),*>)? {
            fn hash<H: $crate::Hasher<u64>>(&self, state: &mut H) {
                struct Wrap<'a, H: $crate::Hasher<u64>>(&'a mut H);
                impl <H: $crate::Hasher<u64>> ::core::hash::Hasher for Wrap<'_, H> {
                    #[inline(always)]
                    fn finish(&self) -> u64 {
                        H::finish(self.0)
                    }

                    impl_hasher_fwd!();
                }
                <Self as ::core::hash::Hash>::hash(self, &mut Wrap(state))
            }
        }
    )* }
}

struct WrapCoreForGen<'a, T, H: Hasher<T>>(&'a mut H, ::core::marker::PhantomData<T>);
impl<T, H: Hasher<T>> ::core::hash::Hasher for WrapCoreForGen<'_, T, H> {
    #[inline(always)]
    fn finish(&self) -> u64 {
        panic!("`core::hash::Hasher::finish` called while calculating generic hash");
    }

    impl_hasher_fwd!();
}

#[macro_export]
/// Implement [`Hash<T>`] for types that already implement `::core::hash::Hash`.
/// This will panic if `::core::hash::Hasher::finish` is called during hashing.
/// You can use [`impl_hash_u64`] instead to only implement [`Hash<u64>`].
macro_rules! impl_hash_gen {
    ($($t:ident $(<
        $($lt0:lifetime $(,$lt:lifetime)*)? $(,)? $($gen:ident $(: $con0:path $(: $con:path)*)?),*
    >)?),* $(,)?) => { $(
        impl<$($($lt0, $($lt,)*)?)? T $($(, $gen: $($con0 $(+ $con)*)?),*)?>
        $crate::Hash<T> for $t $(<$($lt0, $($lt,)*)? $($gen),*>)? {
            fn hash<H: $crate::Hasher<T>>(&self, state: &mut H) {
                <Self as ::core::hash::Hash>::hash(self,
                    &mut $crate::WrapCoreForGen(state, ::core::marker::PhantomData)
                )
            }
        }
    )* }
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

                    impl_hasher_fwd!();
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

macro_rules! make_hasher_writes {
    ($($t:ty { ne:$ne:ident, le:$le:ident, be:$be:ident }),* $(,)?) => { $(
        #[doc = "Writes a single `"]
        #[doc = stringify!($t)]
        #[doc = "` into this hasher in native byte order."]
        #[inline]
        fn $ne(&mut self, i: $t) {
            self.write(&i.to_ne_bytes());
        }

        #[doc = "Writes a single `"]
        #[doc = stringify!($t)]
        #[doc = "` into this hasher in little endian byte order."]
        #[inline]
        fn $le(&mut self, i: $t) {
            self.write(&i.to_le_bytes());
        }

        #[doc = "Writes a single `"]
        #[doc = stringify!($t)]
        #[doc = "` into this hasher in big endian byte order."]
        #[inline]
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
    #[inline]
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
    #[inline]
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

    /// Writes a length prefix into this hasher, as part of being prefix-free.
    #[inline]
    fn write_length_prefix(&mut self, len: usize) {
        self.write_usize(len);
    }

    /// Writes a single str into this hasher.
    #[inline]
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
    fn hash_one<U: Hash<T>>(&self, x: U) -> T {
        let mut hasher = self.build_hasher();
        x.hash(&mut hasher);
        hasher.finish()
    }
}

macro_rules! impl_hash_prim {
    ($($t:ty $(as $u:ty)?: $ne:ident),* $(,)?) => { $(
        impl<T> $crate::Hash<T> for $t {
            #[inline]
            fn hash<H: Hasher<T>>(&self, state: &mut H) {
                state.$ne(*self $(as $u)?)
            }
        }
    )* };
}

impl_hash_prim! {
    u8: write_u8,
    u16: write_u16,
    u32: write_u32,
    u64: write_u64,
    u128: write_u128,
    usize: write_usize,
    i8: write_i8,
    i16: write_i16,
    i32: write_i32,
    i64: write_i64,
    i128: write_i128,
    isize: write_isize,
    bool as u8: write_u8,
    char as u32: write_u32,
}

impl<T> Hash<T> for str {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        state.write_str(self)
    }
}

impl<T, U: Hash<T>> Hash<T> for [U] {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        state.write_length_prefix(self.len());
        Hash::<T>::hash_slice(self, state);
    }
}

impl<T, U: Hash<T>, const N: usize> Hash<T> for [U; N] {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        Hash::<T>::hash(&self[..], state);
    }
}

impl<T, U: ?Sized + Hash<T>> Hash<T> for &U {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T, U: ?Sized + Hash<T>> Hash<T> for &mut U {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T, U: ?Sized> Hash<T> for *const U {
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        <Self as ::core::hash::Hash>::hash(
            self,
            &mut WrapCoreForGen(state, ::core::marker::PhantomData),
        )
    }
}

impl<T, U: ?Sized> Hash<T> for *mut U {
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        <Self as ::core::hash::Hash>::hash(
            self,
            &mut WrapCoreForGen(state, ::core::marker::PhantomData),
        )
    }
}

impl_empty_hash!(());

macro_rules! impl_hash_tuple {
    ($(($($i:ident),+ $(,)?)),* $(,)?) => { $(
        impl<T, $($i: Hash<T>),* + ?Sized> Hash<T> for ($($i,)*) {
            #[inline]
            fn hash<H: Hasher<T>>(&self, state: &mut H) {
                #[allow(non_snake_case)]
                let ($($i,)*) = self;
                $( $i.hash(state); )*
            }
        }
    )* };
}

impl_hash_tuple! {
    (T0),
    (T0, T1),
    (T0, T1, T2),
    (T0, T1, T2, T3),
    (T0, T1, T2, T3, T4),
    (T0, T1, T2, T3, T4, T5),
    (T0, T1, T2, T3, T4, T5, T6),
    (T0, T1, T2, T3, T4, T5, T6, T7),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
}

mod core_impls {
    use super::*;
    use core::{
        cmp::{Ordering, Reverse},
        convert::Infallible,
        ffi::CStr,
        fmt::Error,
        marker::{PhantomData, PhantomPinned},
        mem::{discriminant, transmute, Discriminant, ManuallyDrop},
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
        num::{
            NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
            NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Saturating, Wrapping,
        },
        ops::{
            Bound, ControlFlow, Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
            RangeToInclusive,
        },
        panic::Location,
        pin::Pin,
        ptr::NonNull,
        sync::atomic,
        task::Poll,
        time::Duration,
    };

    impl<T> Hash<T> for Ordering {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            (*self as i8).hash(state)
        }
    }

    impl<T, U: ?Sized> Hash<T> for PhantomData<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, _: &mut H) {}
    }

    impl_hash_gen!(Discriminant<U>);

    impl<T, U: Hash<T>> Hash<T> for ManuallyDrop<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            Hash::<T>::hash(
                unsafe {
                    // # Safety
                    // `ManuallyDrop<T>` is guaranteed to have the same layout and bit validity as T
                    transmute::<&Self, &U>(self)
                },
                state,
            );
        }
    }

    impl_hash_gen!(SocketAddrV4, SocketAddrV6);

    impl<T> Hash<T> for IpAddr {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                IpAddr::V4(v4) => v4.hash(state),
                IpAddr::V6(v6) => v6.hash(state),
            }
        }
    }

    impl<T> Hash<T> for SocketAddr {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                SocketAddr::V4(v4) => v4.hash(state),
                SocketAddr::V6(v6) => v6.hash(state),
            }
        }
    }

    impl<T, I: Hash<T>> Hash<T> for Range<I> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            Hash::<T>::hash(&self.start, state);
            Hash::<T>::hash(&self.end, state);
        }
    }

    // RangeInclusive has private internal state
    impl_hash_gen!(RangeInclusive<I: core::hash::Hash>);

    impl<T, U: Hash<T>> Hash<T> for Bound<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                Bound::Included(x) | Bound::Excluded(x) => Hash::<T>::hash(x, state),
                Bound::Unbounded => (),
            }
        }
    }

    impl<T, B: Hash<T>, C: Hash<T>> Hash<T> for ControlFlow<B, C> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                ControlFlow::Continue(c) => Hash::<T>::hash(c, state),
                ControlFlow::Break(b) => Hash::<T>::hash(b, state),
            }
        }
    }

    impl<T, U: Hash<T>> Hash<T> for Option<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                None => (),
                Some(x) => Hash::<T>::hash(x, state),
            }
        }
    }

    impl_hash_gen!(Location<'a>);

    impl<T, P: Deref<Target = impl Hash<T>>> Hash<T> for Pin<P> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            P::Target::hash(self, state)
        }
    }

    impl<T, U: ?Sized + Hash<T>> Hash<T> for NonNull<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            self.as_ptr().hash(state);
        }
    }

    impl<T, U: Hash<T>, E: Hash<T>> Hash<T> for Result<U, E> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                Ok(x) => Hash::<T>::hash(x, state),
                Err(x) => Hash::<T>::hash(x, state),
            }
        }
    }

    impl<T> Hash<T> for atomic::Ordering {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
        }
    }

    impl<T, U: Hash<T>> Hash<T> for Poll<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                Poll::Ready(x) => Hash::<T>::hash(x, state),
                Poll::Pending => (),
            }
        }
    }

    impl_hash_gen!(Duration);

    impl_empty_hash! {
        Infallible, Error, PhantomPinned, RangeFull
    }

    impl_hash_from_method! {
        [T]
        get {
            NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
            NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
        }
        octets {
            Ipv4Addr, Ipv6Addr,
        }
        to_bytes_with_nul {
            CStr,
        }
    }

    impl_hash_from_field! {
        [T]
        0 {
            Reverse<U: Hash<T>>,
            Saturating<U: Hash<T>>, Wrapping<U: Hash<T>>,
        }
        start {
            RangeFrom<I: Hash<T>>,
        }
        end {
            RangeTo<I: Hash<T>>,
            RangeToInclusive<I: Hash<T>>,
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::*;
    use alloc::{
        boxed::Box,
        collections::{BTreeMap, BTreeSet, LinkedList, VecDeque},
        rc::Rc,
        string::String,
        sync::Arc,
        vec::Vec,
    };

    impl<T, U: ?Sized + Hash<T>> Hash<T> for Box<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T, U: ?Sized + Hasher<T>> Hasher<T> for Box<U> {
        impl_hasher_deref!();
    }

    impl<T, U: ?Sized + Hash<T>> Hash<T> for Rc<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T, U: ?Sized + Hash<T>> Hash<T> for Arc<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T> Hash<T> for String {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T, U: Hash<T>> Hash<T> for Vec<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T, K: Hash<T>, V: Hash<T>> Hash<T> for BTreeMap<K, V> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            for item in self {
                item.hash(state);
            }
        }
    }

    impl_hash_gen!(BTreeSet<K: core::hash::Hash>);

    impl<T, U: Hash<T>> Hash<T> for LinkedList<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            for item in self {
                item.hash(state);
            }
        }
    }

    impl<T, U: Hash<T>> Hash<T> for VecDeque<U> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            for item in self {
                item.hash(state);
            }
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
