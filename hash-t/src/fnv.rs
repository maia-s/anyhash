//! Hasher and collections using the Fnv hashing algorithm.

use core::{fmt::Debug, marker::PhantomData, ops::BitXorAssign};

use crate::{
    impl_core_build_hasher, impl_core_hasher, BuildHasher, EndianIndependentAlgorithm, Hasher,
    HasherWrite,
};

#[cfg(feature = "bnum")]
use bnum::types::{U1024, U128, U256, U512};

impl_core_build_hasher!(Fnv1aBuildHasher<u64>; Fnv1aDefaultBuildHasher);
impl_core_hasher!(Fnv1a<u64>);

/// [`BuildHasher`] implementation for the [`Fnv1a`] hasher.
/// If you don't need support for using custom seeds, use the zero sized
/// [`Fnv1aDefaultBuildHasher`] instead.
pub type Fnv1aBuildHasher<T> = FnvBuildHasher<T, V1A>;

/// [`BuildHasher`] implementation for the [`Fnv`] hasher.
/// If you don't need support for using custom seeds, use the zero sized
/// [`FnvDefaultBuildHasher`] instead.
#[derive(Clone, Debug)]
pub struct FnvBuildHasher<T, V>(T, PhantomData<fn() -> V>);

impl<T: Type, V: Version> FnvBuildHasher<T, V> {
    /// Create a [`BuildHasher`] for [`Fnv`] using the default seed.
    #[inline]
    pub const fn new() -> Self {
        Self::with_seed(Fnv1a::<T>::OFFSET_BASIS)
    }

    /// Create a [`BuildHasher`] for [`Fnv`] with a custom seed.
    #[inline]
    pub const fn with_seed(seed: T) -> Self {
        Self(seed, PhantomData)
    }
}

impl<T: Type, V: Version> Default for FnvBuildHasher<T, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Type, V: Version> BuildHasher<T> for FnvBuildHasher<T, V> {
    type Hasher = Fnv<T, V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::with_seed(self.0)
    }
}

/// [`BuildHasher`] implementation for the [`Fnv`] hasher using the default seed (zero sized).
pub type Fnv1aDefaultBuildHasher = FnvDefaultBuildHasher<V1A>;

/// [`BuildHasher`] implementation for the [`Fnv`] hasher using the default seed (zero sized).
#[derive(Clone, Debug, Default)]
pub struct FnvDefaultBuildHasher<V: Version>(PhantomData<fn() -> V>);

impl<V: Version> FnvDefaultBuildHasher<V> {
    /// Create a [`BuildHasher`] for [`Fnv`] using the default seed.
    #[inline]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Type, V: Version> BuildHasher<T> for FnvDefaultBuildHasher<V> {
    type Hasher = Fnv<T, V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::new()
    }
}

#[cfg(feature = "std")]
/// `HashMap` from `std` configured to use the [`Fnv1a64`] hasher.
pub type Fnv1aHashMap<K, V> = std::collections::HashMap<K, V, Fnv1aBuildHasher<u64>>;

#[cfg(feature = "std")]
/// `HashMap` from `std` configured to use the [`Fnv1a64`] hasher with the default seed.
pub type Fnv1aDefaultHashMap<K, V> = std::collections::HashMap<K, V, Fnv1aDefaultBuildHasher>;

#[cfg(feature = "std")]
/// `HashSet` from `std` configured to use the [`Fnv1a64`] hasher.
pub type Fnv1aHashSet<T> = std::collections::HashSet<T, Fnv1aBuildHasher<u64>>;

#[cfg(feature = "std")]
/// `HashSet` from `std` configured to use the [`Fnv1a64`] hasher with the default seed.
pub type Fnv1aDefaultHashSet<T> = std::collections::HashSet<T, Fnv1aDefaultBuildHasher>;

/// Hasher using the Fnv1a 32-bit algorithm.
pub type Fnv1a32 = Fnv1a<u32>;

/// Hasher using the Fnv1a 64-bit algorithm.
pub type Fnv1a64 = Fnv1a<u64>;

/// Hasher using the Fnv1a 128-bit algorithm.
pub type Fnv1a128 = Fnv1a<u128>;

#[cfg(feature = "bnum")]
/// Hasher using the Fnv1a 256-bit algorithm.
pub type Fnv1a256 = Fnv1a<U256>;

#[cfg(feature = "bnum")]
/// Hasher using the Fnv1a 512-bit algorithm.
pub type Fnv1a512 = Fnv1a<U512>;

#[cfg(feature = "bnum")]
/// Hasher using the Fnv1a 1024-bit algorithm.
pub type Fnv1a1024 = Fnv1a<U1024>;

use sealed::{Type, Version};
mod sealed {

    use super::*;

    /// Configuration trait for the Fnv1a hashers.
    pub trait Type: Copy + From<u8> + BitXorAssign {
        /// Offset basis.
        const OFFSET_BASIS: Self;

        /// Prime.
        const PRIME: Self;

        /// Wrapping multiply.
        fn wrapping_mul(self, rhs: Self) -> Self;
    }

    /// Fnv version configuration.
    pub trait Version: Clone + Debug + Default {
        /// Whether to xor before multiply.
        const XOR_BEFORE_MULTIPLY: bool;
    }
}

impl Type for u32 {
    const OFFSET_BASIS: Self = 0x811c9dc5;
    const PRIME: Self = 0x01000193;

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

impl Type for u64 {
    const OFFSET_BASIS: Self = 0xcbf29ce484222325;
    const PRIME: Self = 0x100000001b3;

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

impl Type for u128 {
    const OFFSET_BASIS: Self = 0x6c62272e07bb014262b821756295c58d;
    const PRIME: Self = 0x0000000001000000000000000000013b;

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

#[cfg(feature = "bnum")]
impl Type for U128 {
    const OFFSET_BASIS: Self = Self::from_digits([0x62b821756295c58d, 0x6c62272e07bb0142]);
    const PRIME: Self = Self::from_digits([0x13b, 0x1000000]);

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

#[cfg(feature = "bnum")]
impl Type for U256 {
    const OFFSET_BASIS: Self = Self::from_digits([
        0x1023b4c8caee0535,
        0xc8b1536847b6bbb3,
        0x2d98c384c4e576cc,
        0xdd268dbcaac55036,
    ]);
    const PRIME: Self = Self::from_digits([0x163, 0, 0x10000000000, 0]);

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

#[cfg(feature = "bnum")]
impl Type for U512 {
    const OFFSET_BASIS: Self = Self::from_digits([
        0xac982aac4afe9fd9,
        0x182036415f56e34b,
        0x2ea79bc942dbe7ce,
        0xe948f68a34c192f6,
        0x0000000000000d21,
        0xac87d059c9000000,
        0xdca1e50f309990ac,
        0xb86db0b1171f4416,
    ]);
    const PRIME: Self = Self::from_digits([0x157, 0, 0, 0, 0, 0x1000000, 0, 0]);

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

#[cfg(feature = "bnum")]
impl Type for U1024 {
    const OFFSET_BASIS: Self = Self::from_digits([
        0xaff4b16c71ee90b3,
        0x6bde8cc9c6a93b21,
        0x555f256cc005ae55,
        0xeb6e73802734510a,
        0x000000000004c6d7,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x9a21d90000000000,
        0x6c3bf34eda3674da,
        0x4b29fc4223fdada1,
        0x32e56d5a591028b7,
        0x005f7a76758ecc4d,
        0x0000000000000000,
    ]);

    const PRIME: Self = Self::from_digits([
        0x18D,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0x10000000000,
        0,
        0,
        0,
        0,
        0,
    ]);

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

/// Selector for Fnv1.
#[derive(Clone, Copy, Debug, Default)]
pub struct V1;

impl Version for V1 {
    const XOR_BEFORE_MULTIPLY: bool = false;
}

/// Selector for Fnv1a.
#[derive(Clone, Copy, Debug, Default)]
pub struct V1A;

impl Version for V1A {
    const XOR_BEFORE_MULTIPLY: bool = true;
}

/// Hasher using the Fnv1a algorithm.
pub type Fnv1a<T> = Fnv<T, V1A>;

#[derive(Clone)]
/// Hasher using a variant of the Fnv algorithm.
pub struct Fnv<T, V>(T, PhantomData<fn() -> V>);

impl<T: Type, V: Version> Fnv<T, V> {
    const OFFSET_BASIS: T = T::OFFSET_BASIS;
    const PRIME: T = T::PRIME;

    /// Create a new `Fnv` hasher using the default seed.
    #[inline]
    pub const fn new() -> Self {
        Self::with_seed(Self::OFFSET_BASIS)
    }

    /// Create a new `Fnv` hasher with a custom seed.
    #[inline]
    pub const fn with_seed(seed: T) -> Self {
        Self(seed, PhantomData)
    }
}

impl<T: Type, V: Version> EndianIndependentAlgorithm for Fnv<T, V> {}

impl<T: Type, V: Version> Default for Fnv<T, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Type, V: Version> HasherWrite for Fnv<T, V> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if V::XOR_BEFORE_MULTIPLY {
                self.0 ^= byte.into();
                self.0 = self.0.wrapping_mul(Self::PRIME);
            } else {
                self.0 = self.0.wrapping_mul(Self::PRIME);
                self.0 ^= byte.into();
            }
        }
    }
}

impl<T: Type, V: Version> Hasher<T> for Fnv<T, V> {
    #[inline]
    fn finish(&self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use core::any::type_name;

    use super::*;
    use crate::{tests::RawBytes, BuildHasher, Hash};

    #[test]
    fn fnv0() {
        fn test<T: Type + Debug + Eq + From<u8>>() {
            let hash = FnvBuildHasher::<T, V1>::with_seed(0.into())
                .hash_one(RawBytes(br"chongo <Landon Curt Noll> /\../\"));
            assert_eq!(
                hash,
                T::OFFSET_BASIS,
                "fnv0 test failed for {}",
                type_name::<T>()
            );
        }

        test::<u32>();
        test::<u64>();
        test::<u128>();

        #[cfg(feature = "bnum")]
        test::<U128>();
        #[cfg(feature = "bnum")]
        test::<U256>();
        #[cfg(feature = "bnum")]
        test::<U512>();
        #[cfg(feature = "bnum")]
        test::<U1024>();
    }

    fn fnv1a_default_seed<T: Hash>(x: T) -> u64 {
        Fnv1aDefaultBuildHasher::default().hash_one(x)
    }

    fn fnv1a_custom_seed<T: Hash>(x: T) -> u64 {
        Fnv1aBuildHasher::with_seed(0x55555555_55555555).hash_one(x)
    }

    #[test]
    fn fnv1a_empty_default_seed() {
        assert_eq!(fnv1a_default_seed(()), 0xcbf29ce484222325);
    }

    #[test]
    fn fnv1a_empty_custom_seed() {
        assert_eq!(fnv1a_custom_seed(()), 0x5555555555555555);
    }

    test_bytes_hash! {
        [fnv1a_default_seed]
        a: 0xaf63dc4c8601ec8c,
        ab: 0x89c4407b545986a,
        abc: 0xe71fa2190541574b,
        abcd: 0xfc179f83ee0724dd,
        abcde: 0x6348c52d762364a8,
        abcdef: 0xd80bda3fbe244a0a,
        abcdefg: 0x406e475017aa7737,
        abcdefgh: 0x25da8c1836a8d66d,
        abcdefghi: 0xfb321124e0e3a8cc,
        abcdefghij: 0xb9bbc7aa22d79212,
        abcdefghijk: 0x71a6bf19344de39b,
        abcdefghijkl: 0x6c3aaed3e05a5cb5,
        abcdefghijklm: 0x4213ea06398bc308,
        abcdefghijklmn: 0xd39a0e93c87d0652,
        abcdefghijklmno: 0xbcd021dac7199a7,
        abcdefghijklmnop: 0x7ef46f6c05086855,
        abcdefghijklmnopq: 0xc1c1788c8d48f52c,
        abcdefghijklmnopqr: 0x84b534d412f8eeba,
        abcdefghijklmnopqrs: 0x78d78d5c3cfdbf8b,
        abcdefghijklmnopqrst: 0x540532bba32d3e4d,
        abcdefghijklmnopqrstu: 0xf2136cd645e0b928,
        abcdefghijklmnopqrstuv: 0x37bb4e18bcdafaba,
        abcdefghijklmnopqrstuvw: 0x8e408108e8182a57,
        abcdefghijklmnopqrstuvwx: 0xcfc57122610faddd,
        abcdefghijklmnopqrstuvwxy: 0x1c2ce16aeda40dac,
        abcdefghijklmnopqrstuvwxyz: 0x8450deb1cdc382a2,
        abcdefghijklmnopqrstuvwxyz0: 0x98ecfa20a336de16,
        abcdefghijklmnopqrstuvwxyz01: 0x118b2c75563b7c45,
        abcdefghijklmnopqrstuvwxyz012: 0xaf9026187147e35,
        abcdefghijklmnopqrstuvwxyz0123: 0xb99d11b887d22432,
        abcdefghijklmnopqrstuvwxyz01234: 0x3809228eca133632,
        abcdefghijklmnopqrstuvwxyz012345: 0x4abbbfa15ea4cde5,
        abcdefghijklmnopqrstuvwxyz0123456: 0xa1d47233d209bd89,
        abcdefghijklmnopqrstuvwxyz01234567: 0x5bbcc0de68d69da,
        abcdefghijklmnopqrstuvwxyz012345678: 0x4b859d9ec24aeb06,
        abcdefghijklmnopqrstuvwxyz0123456789: 0x9ef613c4254dbc0d,

        [fnv1a_custom_seed]
        a: 0x555533ffffffc75c,
        ab: 0xff8e99ffff9f8e5a,
        abc: 0xdedde6ff5c1eaadb,
        abcd: 0xd1ba42e9881c228d,
        abcde: 0x7ba29ad247cf5038,
        abcdef: 0xe49d715005458fba,
        abcdefg: 0xbd1767f8f5337487,
        abcdefgh: 0x823a9b08a66fb21d,
        abcdefghi: 0xb947e3b2cfcc3b1c,
        abcdefghij: 0xa1635ed718090982,
        abcdefghijk: 0x44e4107dd75bd6eb,
        abcdefghijkl: 0x6b5e8cd4f10d8765,
        abcdefghijklm: 0x7f3055d599fc7298,
        abcdefghijklmn: 0x1b94cff4a7f75802,
        abcdefghijklmno: 0xd535c9b9694b4137,
        abcdefghijklmnop: 0x95a70d0deadfeba5,
        abcdefghijklmnopq: 0x2ac702a61a7db93c,
        abcdefghijklmnopqr: 0x2de2ce3f03a1df8a,
        abcdefghijklmnopqrs: 0x9a446e132c0f941b,
        abcdefghijklmnopqrst: 0x31db7993de79389d,
        abcdefghijklmnopqrstu: 0x31287e4307fbb238,
        abcdefghijklmnopqrstuv: 0x8380d9e690affa8a,
        abcdefghijklmnopqrstuvw: 0x23ed3fc7db077be7,
        abcdefghijklmnopqrstuvwx: 0x139eff992db70f2d,
        abcdefghijklmnopqrstuvwxy: 0xe3ba548ae0f0bbc,
        abcdefghijklmnopqrstuvwxyz: 0x3e65a07fc3910172,
        abcdefghijklmnopqrstuvwxyz0: 0x97b0fb194f652326,
        abcdefghijklmnopqrstuvwxyz01: 0x26ddc301e8daa015,
        abcdefghijklmnopqrstuvwxyz012: 0xe572833eab7e2245,
        abcdefghijklmnopqrstuvwxyz0123: 0x5fb7797d67548e82,
        abcdefghijklmnopqrstuvwxyz01234: 0xf952261694ae7f42,
        abcdefghijklmnopqrstuvwxyz012345: 0x55162f5ea4829735,
        abcdefghijklmnopqrstuvwxyz0123456: 0x174980d189e69a19,
        abcdefghijklmnopqrstuvwxyz01234567: 0x7880120d52d7fc2a,
        abcdefghijklmnopqrstuvwxyz012345678: 0x999abea3c5015296,
        abcdefghijklmnopqrstuvwxyz0123456789: 0x3449f47c13f7f5d,
    }
}
