use crate::{impl_hash, Hash, Hasher, HasherWrite};

macro_rules! impl_hasher_t_deref {
    () => {
        #[inline]
        fn finish(&self) -> HT {
            (**self).finish()
        }
    };
}

macro_rules! impl_hasher_deref_writes {
    ($($t:ty: $fn:ident),* $(,)?) => { $(
        #[inline]
        fn $fn(&mut self, i: $t) {
            (**self).$fn(i);
        }
    )* }
}

macro_rules! impl_hasher_deref {
    () => {
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
            u16: write_u16,
            u32: write_u32,
            u64: write_u64,
            u128: write_u128,
            usize: write_usize,

            i16: write_i16,
            i32: write_i32,
            i64: write_i64,
            i128: write_i128,
            isize: write_isize,
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
        impl Hash for $t {
            #[inline]
            fn hash<H: HasherWrite>(&self, _: &mut H) {}
        }
    )* };
}

macro_rules! impl_hash_from_method {
    ($($m:ident { $($t:ty),* $(,)? })*) => { $($(
        impl Hash for $t {
            #[inline]
            fn hash<H: HasherWrite>(&self, state: &mut H) {
                Hash::hash(&self.$m(), state)
            }
        }
    )*)* };
}

macro_rules! impl_hash_from_field {
    ($($n:tt { $($t:ident $(<
        $($gen:ident $(: $con0:path $(: $con:path)*)?),+
    >)?),* $(,)? })*) => { $($(
        impl<$($($gen $(: $con0 $(+ $con)*)?),*)?> Hash for $t $(<$($gen),*>)? {
            #[inline]
            fn hash<H: HasherWrite>(&self, state: &mut H) {
                Hash::hash(&self.$n, state)
            }
        }
    )*)* };
}

impl<HT, H: Hasher<HT> + ?Sized> Hasher<HT> for &mut H {
    impl_hasher_t_deref!();
}

impl<H: HasherWrite + ?Sized> HasherWrite for &mut H {
    impl_hasher_deref!();
}

macro_rules! impl_hash_prim {
    ($($t:ty $(as $u:ty)?: $ne:ident),* $(,)?) => { $(
        impl $crate::Hash for $t {
            #[inline]
            fn hash<H: HasherWrite>(&self, state: &mut H) {
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

impl Hash for str {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        state.write_str(self)
    }
}

impl<T: Hash> Hash for [T] {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        state.write_length_prefix(self.len());
        Hash::hash_slice(self, state);
    }
}

impl<T: Hash, const N: usize> Hash for [T; N] {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        Hash::hash(&self[..], state);
    }
}

impl<T: ?Sized + Hash> Hash for &T {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized + Hash> Hash for &mut T {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized> Hash for *const T {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        <Self as ::core::hash::Hash>::hash(
            self,
            &mut crate::internal::WrapHasherWriteForCore::new(state),
        )
    }
}

impl<T: ?Sized> Hash for *mut T {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        <Self as ::core::hash::Hash>::hash(
            self,
            &mut crate::internal::WrapHasherWriteForCore::new(state),
        )
    }
}

macro_rules! impl_hash_tuple_and_fn {
    ($(($($i:ident),+ $(,)?)),* $(,)?) => { $(
        impl<$($i: Hash),* + ?Sized> Hash for ($($i,)*) {
            #[inline]
            fn hash<H: HasherWrite>(&self, state: &mut H) {
                #[allow(non_snake_case)]
                let ($($i,)*) = self;
                $( $i.hash(state); )*
            }
        }

        impl<R $(, $i)*> Hash for fn($($i),*) -> R {
            #[inline]
            fn hash<H: HasherWrite>(&self, state: &mut H) {
                (*self as usize).hash(state);
            }
        }

        impl<R $(, $i)*> Hash for extern "C" fn($($i),*) -> R {
            #[inline]
            fn hash<H: HasherWrite>(&self, state: &mut H) {
                (*self as usize).hash(state);
            }
        }
    )* };
}

impl_empty_hash!(());

impl<R> Hash for fn() -> R {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
        (*self as usize).hash(state);
    }
}

impl<R> Hash for extern "C" fn() -> R {
    #[inline]
    fn hash<H: HasherWrite>(&self, state: &mut H) {
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

    impl_hash!(Layout; TypeId);

    impl Hash for Ordering {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            (*self as i8).hash(state)
        }
    }

    impl<T: ?Sized> Hash for PhantomData<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, _: &mut H) {}
    }

    impl_hash!(impl<T> Discriminant<T>);

    impl<T: ?Sized + Hash> Hash for ManuallyDrop<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            Hash::hash(
                unsafe {
                    // # Safety
                    // `ManuallyDrop<T>` is guaranteed to have the same layout and bit validity as T
                    transmute::<&Self, &T>(self)
                },
                state,
            );
        }
    }

    impl<I: Hash> Hash for Range<I> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            Hash::hash(&self.start, state);
            Hash::hash(&self.end, state);
        }
    }

    // RangeInclusive has private internal state
    impl_hash!(impl<I: core::hash::Hash> RangeInclusive<I>);

    impl<T: Hash> Hash for Bound<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                Bound::Included(x) | Bound::Excluded(x) => Hash::hash(x, state),
                Bound::Unbounded => (),
            }
        }
    }

    impl<B: Hash, C: Hash> Hash for ControlFlow<B, C> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                ControlFlow::Continue(c) => Hash::hash(c, state),
                ControlFlow::Break(b) => Hash::hash(b, state),
            }
        }
    }

    impl<T: Hash> Hash for Option<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                None => (),
                Some(x) => Hash::hash(x, state),
            }
        }
    }

    impl_hash!(Location<'_>);

    impl<P: Deref<Target = impl Hash>> Hash for Pin<P> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            P::Target::hash(self, state)
        }
    }

    impl<T: ?Sized + Hash> Hash for NonNull<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            self.as_ptr().hash(state);
        }
    }

    impl<T: Hash, E: Hash> Hash for Result<T, E> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                Ok(x) => Hash::hash(x, state),
                Err(x) => Hash::hash(x, state),
            }
        }
    }

    impl Hash for atomic::Ordering {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            discriminant(self).hash(state);
        }
    }

    impl<T: Hash> Hash for Poll<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            discriminant(self).hash(state);
            match self {
                Poll::Ready(x) => Hash::hash(x, state),
                Poll::Pending => (),
            }
        }
    }

    impl_hash!(Duration);

    impl_empty_hash! {
        Infallible, Error, PhantomPinned, RangeFull
    }

    impl_hash_from_method! {
        get {
            NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
            NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
        }
        to_bytes_with_nul {
            CStr,
        }
    }

    impl_hash_from_field! {
        0 {
            Reverse<T: Hash>,
            Saturating<T: Hash>, Wrapping<T: Hash>,
        }
        start {
            RangeFrom<I: Hash>,
        }
        end {
            RangeTo<I: Hash>,
            RangeToInclusive<I: Hash>,
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

    impl<B: ?Sized + Hash + ToOwned> Hash for Cow<'_, B> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            Hash::hash(&**self, state)
        }
    }

    impl<T: ?Sized + Hash> Hash for Box<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<HT, T: ?Sized + Hasher<HT>> Hasher<HT> for Box<T> {
        impl_hasher_t_deref!();
    }

    impl<T: ?Sized + HasherWrite> HasherWrite for Box<T> {
        impl_hasher_deref!();
    }

    impl<T: ?Sized + Hash> Hash for Rc<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T: ?Sized + Hash> Hash for Arc<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl Hash for String {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<T: Hash> Hash for Vec<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }

    impl<K: Hash, V: Hash> Hash for BTreeMap<K, V> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            for item in self {
                item.hash(state);
            }
        }
    }

    impl_hash!(impl<T: core::hash::Hash> BTreeSet<T>);

    impl<T: Hash> Hash for LinkedList<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            for item in self {
                item.hash(state);
            }
        }
    }

    impl<T: Hash> Hash for VecDeque<T> {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            for item in self {
                item.hash(state);
            }
        }
    }

    impl Hash for CString {
        #[inline]
        fn hash<H: HasherWrite>(&self, state: &mut H) {
            Hash::hash(&**self, state);
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use super::*;
    use std::{
        ffi::{OsStr, OsString},
        fs::FileType,
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
        path::{Component, Path, PathBuf, Prefix, PrefixComponent},
        thread::ThreadId,
        time::{Instant, SystemTime},
    };

    impl_hash!(
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
        SocketAddr;
        SocketAddrV4;
        SocketAddrV6;
        IpAddr;
        Ipv4Addr;
        Ipv6Addr;
    );
}

#[cfg(feature = "bnum")]
mod bnum_impls {
    use bnum::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};

    use super::*;

    macro_rules! impl_buint {
        ($($id:ident),*) => { $(
            impl<const N: usize> Hash for $id<N> {
                fn hash<H: HasherWrite>(&self, state: &mut H) {
                    self.digits().hash(state)
                }
            }
        )* };
    }

    macro_rules! impl_bint {
        ($($id:ident),*) => { $(
            impl<const N: usize> Hash for $id<N> {
                fn hash<H: HasherWrite>(&self, state: &mut H) {
                    self.to_bits().hash(state)
                }
            }
        )* };
    }

    impl_buint!(BUint, BUintD8, BUintD16, BUintD32);
    impl_bint!(BInt, BIntD8, BIntD16, BIntD32);
}
