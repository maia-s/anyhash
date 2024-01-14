use crate::{Hasher, HasherWrite};

use core::marker::PhantomData;

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
}

impl<H: ::core::hash::Hasher> HasherWrite for WrapCoreForU64<'_, H> {
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

#[cfg(feature = "bytemuck")]
pub(crate) use bm::*;

#[cfg(feature = "bytemuck")]
pub(crate) mod bm {
    use bytemuck::{cast_mut, cast_ref, Pod};

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

    #[derive(Clone, Copy, Default)]
    #[repr(transparent)]
    pub(crate) struct Buffer<N: ConstValue>(N::ArrayU64);

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
