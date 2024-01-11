use crate::{impl_hash_t, Hash, Hasher};

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

impl<T, H: Hasher<T> + ?Sized> Hasher<T> for &mut H {
    impl_hasher_deref!();
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
        <Self as ::core::hash::Hash>::hash(self, &mut crate::internal::WrapCoreForT::new(state))
    }
}

impl<T, U: ?Sized> Hash<T> for *mut U {
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        <Self as ::core::hash::Hash>::hash(self, &mut crate::internal::WrapCoreForT::new(state))
    }
}

macro_rules! impl_hash_tuple_and_fn {
    ($(($($i:ident),+ $(,)?)),* $(,)?) => { $(
        impl<T, $($i: Hash<T>),* + ?Sized> Hash<T> for ($($i,)*) {
            #[inline]
            fn hash<H: Hasher<T>>(&self, state: &mut H) {
                #[allow(non_snake_case)]
                let ($($i,)*) = self;
                $( $i.hash(state); )*
            }
        }

        impl<T, R $(, $i)*> Hash<T> for fn($($i),*) -> R {
            #[inline]
            fn hash<H: Hasher<T>>(&self, state: &mut H) {
                (*self as usize).hash(state);
            }
        }

        impl<T, R $(, $i)*> Hash<T> for extern "C" fn($($i),*) -> R {
            #[inline]
            fn hash<H: Hasher<T>>(&self, state: &mut H) {
                (*self as usize).hash(state);
            }
        }
    )* };
}

impl_empty_hash!(());

impl<T, R> Hash<T> for fn() -> R {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        (*self as usize).hash(state);
    }
}

impl<T, R> Hash<T> for extern "C" fn() -> R {
    #[inline]
    fn hash<H: Hasher<T>>(&self, state: &mut H) {
        (*self as usize).hash(state);
    }
}

impl_hash_tuple_and_fn! {
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
        alloc::Layout,
        any::TypeId,
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

    impl_hash_t!(Layout; TypeId);

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

    impl_hash_t!(impl<U> Discriminant<U>);

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

    impl_hash_t!(SocketAddrV4; SocketAddrV6);

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
    impl_hash_t!(impl<I: core::hash::Hash> RangeInclusive<I>);

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

    impl_hash_t!(Location<'_>);

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

    impl_hash_t!(Duration);

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
        borrow::{Cow, ToOwned},
        boxed::Box,
        collections::{BTreeMap, BTreeSet, LinkedList, VecDeque},
        ffi::CString,
        rc::Rc,
        string::String,
        sync::Arc,
        vec::Vec,
    };

    impl<T, B: ?Sized + Hash<T> + ToOwned> Hash<T> for Cow<'_, B> {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            Hash::<T>::hash(&**self, state)
        }
    }

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

    impl_hash_t!(impl<T: core::hash::Hash> BTreeSet<T>);

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

    impl<T> Hash<T> for CString {
        #[inline]
        fn hash<H: Hasher<T>>(&self, state: &mut H) {
            Hash::<T>::hash(&**self, state);
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use super::*;
    use std::{
        ffi::{OsStr, OsString},
        fs::FileType,
        path::{Component, Path, PathBuf, Prefix, PrefixComponent},
        thread::ThreadId,
        time::{Instant, SystemTime},
    };

    impl_hash_t!(
        OsStr;
        OsString;
        FileType;
        Path;
        PathBuf;
        PrefixComponent<'_>;
        Component<'_>;
        Prefix<'_>;
        ThreadId;
        Instant;
        SystemTime;
    );
}