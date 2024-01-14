//! Hasher and collections using the Xxh64 hashing algorithm.

// based on the spec at https://github.com/Cyan4973/xxHash/blob/dev/doc/xxhash_spec.md

use crate::{
    impl_core_build_hasher, impl_core_hasher,
    internal::{Buffer, N4},
    BuildHasher, BuildHasherDefault, EndianIndependentAlgorithm, Hasher, HasherWrite,
};

impl_core_build_hasher!(Xxh64BuildHasher);
impl_core_hasher!(Xxh64);

/// [`BuildHasher`] implementation for the [`Xxh64`] hasher.
/// If you don't need support for using custom seeds, use the zero sized
/// [`Xxh64BuildHasherDefault`] instead.
#[derive(Clone, Debug)]
pub struct Xxh64BuildHasher(u64);

impl Xxh64BuildHasher {
    /// Create a [`BuildHasher`] for [`Xxh64`] using the default seed.
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create a [`BuildHasher`] for [`Xxh64`] with a custom seed.
    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
        Self(seed)
    }
}

impl BuildHasher<u64> for Xxh64BuildHasher {
    type Hasher = Xxh64;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::with_seed(self.0)
    }
}

impl Default for Xxh64BuildHasher {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// [`BuildHasher`] implementation for the [`Xxh64`] hasher using the default seed (zero sized).
pub type Xxh64BuildHasherDefault = BuildHasherDefault<Xxh64>;

#[cfg(feature = "std")]
/// `HashMap` from `std` configured to use the [`Xxh64`] hasher.
pub type XXh64HashMap<K, V> = std::collections::HashMap<K, V, Xxh64BuildHasher>;

#[cfg(feature = "std")]
/// `HashMap` from `std` configured to use the [`Xxh64`] hasher with the default seed.
pub type XXh64HashMapDefault<K, V> = std::collections::HashMap<K, V, Xxh64BuildHasherDefault>;

#[cfg(feature = "std")]
/// `HashSet` from `std` configured to use the [`Xxh64`] hasher.
pub type XXh64HashSet<T> = std::collections::HashSet<T, Xxh64BuildHasher>;

#[cfg(feature = "std")]
/// `HashSet` from `std` configured to use the [`Xxh64`] hasher with the default seed.
pub type XXh64HashSetDefault<T> = std::collections::HashSet<T, Xxh64BuildHasherDefault>;

/// Hasher using the Xxh64 algorithm.
#[derive(Clone)]
pub struct Xxh64 {
    acc: [u64; 4],
    buffer: Buffer<N4>,
    buffer_len: usize,
    total_len: u64,
}

impl Xxh64 {
    const PRIME64_1: u64 = 0x9e3779b185ebca87;
    const PRIME64_2: u64 = 0xc2b2ae3d27d4eb4f;
    const PRIME64_3: u64 = 0x165667b19e3779f9;
    const PRIME64_4: u64 = 0x85ebca77c2b2ae63;
    const PRIME64_5: u64 = 0x27d4eb2f165667c5;

    /// Create a new `Xxh64` hasher using the default seed.
    #[inline]
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Create a new `Xxh64` hasher with a custom `seed`.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            acc: [
                seed.wrapping_add(Self::PRIME64_1)
                    .wrapping_add(Self::PRIME64_2),
                seed.wrapping_add(Self::PRIME64_2),
                seed,
                seed.wrapping_sub(Self::PRIME64_1),
            ],
            buffer: Buffer::new(),
            buffer_len: 0,
            total_len: 0,
        }
    }

    fn fill_buffer(&mut self, bytes: &mut &[u8]) -> bool {
        let n = bytes.len().min(32 - self.buffer_len);
        let take;
        (take, *bytes) = bytes.split_at(n);
        self.buffer.as_bytes_mut()[self.buffer_len..self.buffer_len + n].copy_from_slice(take);
        self.buffer_len += n;
        self.buffer_len == 32
    }

    #[inline(always)]
    const fn round(acc: u64, lane: u64) -> u64 {
        acc.wrapping_add(lane.wrapping_mul(Self::PRIME64_2))
            .rotate_left(31)
            .wrapping_mul(Self::PRIME64_1)
    }

    #[inline(always)]
    const fn merge_accumulator(acc: u64, acc_n: u64) -> u64 {
        (acc ^ Self::round(0, acc_n))
            .wrapping_mul(Self::PRIME64_1)
            .wrapping_add(Self::PRIME64_4)
    }
}

impl EndianIndependentAlgorithm for Xxh64 {}

impl Default for Xxh64 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl HasherWrite for Xxh64 {
    fn write(&mut self, mut bytes: &[u8]) {
        self.total_len += bytes.len() as u64;
        while !bytes.is_empty() {
            if self.fill_buffer(&mut bytes) {
                self.buffer_len = 0;
                for (acc, &lane) in self.acc.iter_mut().zip(self.buffer.as_u64s().iter()) {
                    *acc = Self::round(*acc, lane.to_le());
                }
            }
        }
    }
}

impl Hasher<u64> for Xxh64 {
    fn finish(&self) -> u64 {
        let acc = if self.total_len < 32 {
            // acc[2] contains the original seed, because the buffer wasn't filled
            self.acc[2].wrapping_add(Self::PRIME64_5)
        } else {
            let mut acc = self.acc[0]
                .rotate_left(1)
                .wrapping_add(self.acc[1].rotate_left(7))
                .wrapping_add(self.acc[2].rotate_left(12))
                .wrapping_add(self.acc[3].rotate_left(18));
            acc = Self::merge_accumulator(acc, self.acc[0]);
            acc = Self::merge_accumulator(acc, self.acc[1]);
            acc = Self::merge_accumulator(acc, self.acc[2]);
            Self::merge_accumulator(acc, self.acc[3])
        };

        let mut acc = acc.wrapping_add(self.total_len);

        let u64s = self.buffer_len / 8;
        for &lane in self.buffer.as_u64s().iter().take(u64s) {
            acc = (acc ^ Self::round(0, lane.to_le()))
                .rotate_left(27)
                .wrapping_mul(Self::PRIME64_1)
                .wrapping_add(Self::PRIME64_4);
        }

        let mut bi = u64s * 8;
        if self.buffer_len - bi >= 4 {
            let lane =
                u32::from_le_bytes(self.buffer.as_bytes()[bi..bi + 4].try_into().unwrap()) as u64;
            bi += 4;
            acc = (acc ^ lane.wrapping_mul(Self::PRIME64_1))
                .rotate_left(23)
                .wrapping_mul(Self::PRIME64_2)
                .wrapping_add(Self::PRIME64_3);
        }

        for &byte in &self.buffer.as_bytes()[bi..self.buffer_len] {
            let lane = byte as u64;
            acc = (acc ^ lane.wrapping_mul(Self::PRIME64_5))
                .rotate_left(11)
                .wrapping_mul(Self::PRIME64_1);
        }

        acc = (acc ^ (acc >> 33)).wrapping_mul(Self::PRIME64_2);
        acc = (acc ^ (acc >> 29)).wrapping_mul(Self::PRIME64_3);
        acc ^ (acc >> 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildHasher, Hash};

    fn default_seed<T: Hash>(x: T) -> u64 {
        Xxh64BuildHasherDefault::new().hash_one(x)
    }

    fn custom_seed<T: Hash>(x: T) -> u64 {
        Xxh64BuildHasher::with_seed(0x55555555_55555555).hash_one(x)
    }

    #[test]
    fn empty_default_seed() {
        assert_eq!(default_seed(()), 0xef46db3751d8e999);
    }

    #[test]
    fn empty_custom_seed() {
        assert_eq!(custom_seed(()), 0x28e7a0126181c619);
    }

    test_bytes_hash! {
        [default_seed]
        a: 0xd24ec4f1a98c6e5b,
        ab: 0x65f708ca92d04a61,
        abc: 0x44bc2cf5ad770999,
        abcd: 0xde0327b0d25d92cc,
        abcde: 0x7e3670c0c8dc7eb,
        abcdef: 0xfa8afd82c423144d,
        abcdefg: 0x1860940e2902822d,
        abcdefgh: 0x3ad351775b4634b7,
        abcdefghi: 0x27f1a34fdbb95e13,
        abcdefghij: 0xd6287a1de5498bb2,
        abcdefghijk: 0x814e257441cf78e0,
        abcdefghijkl: 0x4b09b7d3a233d4b3,
        abcdefghijklm: 0x934adbc0ebc51325,
        abcdefghijklmn: 0xd66d2a9c05576b14,
        abcdefghijklmno: 0x2e1218a2b1375068,
        abcdefghijklmnop: 0x71ce8137ca2dd53d,
        abcdefghijklmnopq: 0x8feff49d8f62f402,
        abcdefghijklmnopqr: 0x6fa4f734e2143ba7,
        abcdefghijklmnopqrs: 0xb95bae7304a854af,
        abcdefghijklmnopqrst: 0xfccc974985dbdc9e,
        abcdefghijklmnopqrstu: 0xfeb122ce2f6dbe1,
        abcdefghijklmnopqrstuv: 0x632cfeac07d58c73,
        abcdefghijklmnopqrstuvw: 0xcf41cc59032e08aa,
        abcdefghijklmnopqrstuvwx: 0xbec95e34669983b,
        abcdefghijklmnopqrstuvwxy: 0xb190b61ba94f20d8,
        abcdefghijklmnopqrstuvwxyz: 0xcfe1f278fa89835c,
        abcdefghijklmnopqrstuvwxyz0: 0xae89c28aaf450c35,
        abcdefghijklmnopqrstuvwxyz01: 0xebbcfd97aa17f75d,
        abcdefghijklmnopqrstuvwxyz012: 0xd7768c31980fd53,
        abcdefghijklmnopqrstuvwxyz0123: 0xab785e0951df0530,
        abcdefghijklmnopqrstuvwxyz01234: 0x16058c7b947da137,
        abcdefghijklmnopqrstuvwxyz012345: 0xbf2cd639b4143b80,
        abcdefghijklmnopqrstuvwxyz0123456: 0x4f89e4082bcbf673,
        abcdefghijklmnopqrstuvwxyz01234567: 0x565de5564aed6b74,
        abcdefghijklmnopqrstuvwxyz012345678: 0xf1911d891becad9f,
        abcdefghijklmnopqrstuvwxyz0123456789: 0x64f23ecf1609b766,

        [custom_seed]
        a: 0x61411dd4ec43e486,
        ab: 0x52ae673d5a2c461f,
        abc: 0xdb99c49d6f09a1b6,
        abcd: 0x19d08ef9bf076c8,
        abcde: 0x19da0bd9e3f6aa43,
        abcdef: 0x7376c9c0eb2975ee,
        abcdefg: 0x16b146c276cac1a8,
        abcdefgh: 0x4f9c528ffadd4fb2,
        abcdefghi: 0xe4ff3d69e6be577d,
        abcdefghij: 0x6431f8b9e835e2e6,
        abcdefghijk: 0x11e5943e40ccdfb7,
        abcdefghijkl: 0x504db7e1dd3280c1,
        abcdefghijklm: 0x6d94d5946431e70a,
        abcdefghijklmn: 0xcf8d4fe41d3b9657,
        abcdefghijklmno: 0x40ea69819a0c7e19,
        abcdefghijklmnop: 0x50eca4d38f7013e6,
        abcdefghijklmnopq: 0x96e0311aa4d94bec,
        abcdefghijklmnopqr: 0x13a1c4ce5195a314,
        abcdefghijklmnopqrs: 0x44911a6ec8652ba,
        abcdefghijklmnopqrst: 0x4e2a9c6fbb4dd441,
        abcdefghijklmnopqrstu: 0x2956fbd2a3957826,
        abcdefghijklmnopqrstuv: 0x9a8d0e8bb7a72439,
        abcdefghijklmnopqrstuvw: 0x8b1fabc53652cc5b,
        abcdefghijklmnopqrstuvwx: 0x4a04e1fc75860c6d,
        abcdefghijklmnopqrstuvwxy: 0x687b63a212964912,
        abcdefghijklmnopqrstuvwxyz: 0x51304ef64f78fcb9,
        abcdefghijklmnopqrstuvwxyz0: 0x39d024caf04a8cd4,
        abcdefghijklmnopqrstuvwxyz01: 0x4b890b92b91f700f,
        abcdefghijklmnopqrstuvwxyz012: 0xf8711e4e7dd048c4,
        abcdefghijklmnopqrstuvwxyz0123: 0x129caa5aa821cdf,
        abcdefghijklmnopqrstuvwxyz01234: 0x9d7b4b91686aec4f,
        abcdefghijklmnopqrstuvwxyz012345: 0xc06ecd739aa8a7d8,
        abcdefghijklmnopqrstuvwxyz0123456: 0x7b644b56e8b2203f,
        abcdefghijklmnopqrstuvwxyz01234567: 0x4bf58a23241496e5,
        abcdefghijklmnopqrstuvwxyz012345678: 0xfcff767d554c3aca,
        abcdefghijklmnopqrstuvwxyz0123456789: 0x1913cbdad3ae2e20,
    }
}
