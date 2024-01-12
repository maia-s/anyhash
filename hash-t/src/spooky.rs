//! Hasher and collections using the SpookyHash algorithm.

// referenced from https://burtleburtle.net/bob/hash/spooky.html

macro_rules! fallthrough_jump_table {
    (switch ($expr:expr) {
        $( $label:lifetime: $pat:pat => $body:expr )*
        => $exitlabel:lifetime:
    }) => {
        fallthrough_jump_table!(@ {
            match $expr {
                $($pat => break $label,)*
            }
        } $($label: $body,)* $exitlabel);
    };

    (@ $build:tt $label:lifetime: $body:expr, $($rest:tt)*) => {
        fallthrough_jump_table!(@ { $label: $build $body } $($rest)*);
    };

    (@ $build:tt $label:lifetime) => {
        $label: $build
    };
}

use core::{marker::PhantomData, slice};

use crate::{impl_core_build_hasher, impl_core_hasher};

use crate::{BuildHasher, Hasher};

impl_core_build_hasher!(impl<V: Version> SpookyBuildHasherV<V>);
impl_core_hasher!(impl<V: Version> SpookyV<V>);

/// Version trait for SpookyHash.
pub trait Version: Clone + core::fmt::Debug + Default {
    /// Version number
    const VERSION: usize;
}

/// Selector for SpookyHash v1.
#[derive(Clone, Debug, Default)]
pub struct V1;

/// Selector for SpookyHash v2.
#[derive(Clone, Debug, Default)]
pub struct V2;

impl Version for V1 {
    const VERSION: usize = 1;
}

impl Version for V2 {
    const VERSION: usize = 2;
}

/// [`BuildHasher`] implementation for the [`Spooky`] v2 hasher.
pub type SpookyBuildHasher = SpookyBuildHasherV<V2>;

/// [`BuildHasher`] implementation for the [`Spooky`] hasher.
#[derive(Clone, Debug)]
pub struct SpookyBuildHasherV<V: Version = V2>(u64, u64, PhantomData<fn() -> V>);

impl<V: Version> SpookyBuildHasherV<V> {
    /// Create a [`BuildHasher`] for [`Spooky`] with the default seed.
    #[inline]
    pub const fn new() -> Self {
        Self::with_seed(0, 0)
    }

    /// Create a [`BuildHasher`] for [`Spooky`] with a custom seed.
    #[inline]
    pub const fn with_seed(seed1: u64, seed2: u64) -> Self {
        Self(seed1, seed2, PhantomData)
    }

    /// Create a [`BuildHasher`] for [`Spooky`] with a custom seed in u128 format.
    #[inline]
    pub const fn with_seed_128(seed: u128) -> Self {
        Self::with_seed(seed as u64, (seed >> 64) as u64)
    }
}

impl<V: Version> BuildHasher<u32> for SpookyBuildHasherV<V> {
    type Hasher = SpookyV<V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::with_seed(self.0, self.1)
    }
}

impl<V: Version> BuildHasher<u64> for SpookyBuildHasherV<V> {
    type Hasher = SpookyV<V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::with_seed(self.0, self.1)
    }
}

impl<V: Version> BuildHasher<u128> for SpookyBuildHasherV<V> {
    type Hasher = SpookyV<V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::with_seed(self.0, self.1)
    }
}

/// [`BuildHasher`] implementation for the [`Spooky`] v2 hasher using the default seed (zero sized).
pub type SpookyDefaultBuildHasher = SpookyDefaultBuildHasherV<V2>;

/// [`BuildHasher`] implementation for the [`Spooky`] hasher using the default seed (zero sized).
#[derive(Clone, Debug, Default)]
pub struct SpookyDefaultBuildHasherV<V: Version = V2>(PhantomData<fn() -> V>);

impl<V: Version> BuildHasher<u32> for SpookyDefaultBuildHasherV<V> {
    type Hasher = SpookyV<V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::new()
    }
}

impl<V: Version> BuildHasher<u64> for SpookyDefaultBuildHasherV<V> {
    type Hasher = SpookyV<V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::new()
    }
}

impl<V: Version> BuildHasher<u128> for SpookyDefaultBuildHasherV<V> {
    type Hasher = SpookyV<V>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::new()
    }
}

#[cfg(feature = "std")]
/// `HashMap` from `std` configured to use the [`Spooky`] v2 hasher.
pub type SpookyHashMap<K, V> = std::collections::HashMap<K, V, SpookyBuildHasher>;

#[cfg(feature = "std")]
/// `HashMap` from `std` configured to use the [`Spooky`] v2 hasher with the default seed.
pub type SpookyDefaultHashMap<K, V> = std::collections::HashMap<K, V, SpookyDefaultBuildHasher>;

#[cfg(feature = "std")]
/// `HashSet` from `std` configured to use the [`Spooky`] v2 hasher.
pub type SpookyHashSet<T> = std::collections::HashSet<T, SpookyBuildHasher>;

#[cfg(feature = "std")]
/// `HashSet` from `std` configured to use the [`Spooky`] v2 hasher with the default seed.
pub type SpookyDefaultHashSet<T> = std::collections::HashSet<T, SpookyDefaultBuildHasher>;

const SC_NUM_VARS: usize = 12;
const SC_BLOCK_SIZE: usize = SC_NUM_VARS * 8;
const SC_BUF_SIZE: usize = SC_BLOCK_SIZE * 2;
const SC_CONST: u64 = 0xdeadbeefdeadbeef;

/// Hasher using the SpookyHash algorithm (v2).
pub type Spooky = SpookyV<V2>;

/// Hasher using the SpookyHash algorithm.
#[derive(Clone)]
pub struct SpookyV<V: Version = V2> {
    data: [u64; 2 * SC_NUM_VARS],
    state: [u64; SC_NUM_VARS],
    length: usize,
    remainder: u8,
    _pd: PhantomData<fn() -> V>,
}

impl<V: Version> SpookyV<V> {
    /// Create a new `Spooky` hasher with the default seed.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: [0; SC_NUM_VARS * 2],
            state: [0; SC_NUM_VARS],
            length: 0,
            remainder: 0,
            _pd: PhantomData,
        }
    }

    /// Create a new `Spooky` hasher with a custom seed.
    #[inline]
    pub fn with_seed(seed1: u64, seed2: u64) -> Self {
        let mut state = [0; SC_NUM_VARS];
        state[0] = seed1;
        state[1] = seed2;
        Self {
            data: [0; SC_NUM_VARS * 2],
            state,
            length: 0,
            remainder: 0,
            _pd: PhantomData,
        }
    }

    /// Create a new `Spooky` hasher with a custom seed in u128 format.
    #[inline]
    pub fn with_seed_128(seed: u128) -> Self {
        Self::with_seed(seed as u64, (seed >> 64) as u64)
    }

    fn mix(data: &[u64; SC_NUM_VARS], s: &mut [u64; SC_NUM_VARS]) {
        macro_rules! mix {
            ($($i:literal, $r:literal);* $(;)?) => { $(
                s[$i] = s[$i].wrapping_add(data[$i]);
                s[($i + 2) % SC_NUM_VARS] ^= s[($i + 10) % SC_NUM_VARS];
                s[($i + 11) % SC_NUM_VARS] ^= s[$i];
                s[$i] = s[$i].rotate_left($r);
                s[($i + 11) % SC_NUM_VARS] = s[($i + 11) % SC_NUM_VARS].wrapping_add(s[($i + 1) % SC_NUM_VARS]);
            )* };
        }
        mix! {
            0, 11; 1, 32; 2, 43; 3, 31;
            4, 17; 5, 28; 6, 39; 7, 57;
            8, 55; 9, 54; 10, 22; 11, 46;
        }
    }

    fn short(&self) -> u128 {
        let length = self.length;

        let mut remainder: usize = length % 32;
        let mut h = [self.state[0], self.state[1], SC_CONST, SC_CONST];

        let mut i = 0;

        if length > 15 {
            i = length / 32 * 4;

            for chunk in self.data[..i].chunks(4) {
                h[2] = h[2].wrapping_add(chunk[0]);
                h[3] = h[3].wrapping_add(chunk[1]);
                Self::short_mix(&mut h);
                h[0] = h[0].wrapping_add(chunk[2]);
                h[1] = h[1].wrapping_add(chunk[3]);
            }

            if remainder >= 16 {
                remainder -= 16;
                h[2] = h[2].wrapping_add(self.data[i]);
                h[3] = h[3].wrapping_add(self.data[i + 1]);
                Self::short_mix(&mut h);
                i += 2;
            }
        }

        if V::VERSION == 1 {
            h[3] = (length as u64).rotate_left(56);
        } else {
            h[3] = h[3].wrapping_add((length as u64).rotate_left(56));
        }

        let data = &self.data[i..i + (remainder + 7) / 8];
        let data_ptr = data.as_ptr();
        let data_len = data.len();
        let (data_u8, data_u32) = unsafe {
            // # Safety
            // Both slices are the same size as the original and have lesser alignment.
            (
                slice::from_raw_parts(data_ptr as *const u8, data_len * 8),
                slice::from_raw_parts(data_ptr as *const u32, data_len * 2),
            )
        };

        fallthrough_jump_table! {
            switch (remainder) {
                'r15: 15 => h[3] = h[3].wrapping_add((data_u8[14] as u64).rotate_left(48))
                'r14: 14 => h[3] = h[3].wrapping_add((data_u8[13] as u64).rotate_left(40))
                'r13: 13 => h[3] = h[3].wrapping_add((data_u8[12] as u64).rotate_left(32))
                'r12: 12 => {
                    h[3] = h[3].wrapping_add(data_u32[2] as u64);
                    h[2] = h[2].wrapping_add(data[0]);
                    break 'done;
                }
                'r11: 11 => h[3] = h[3].wrapping_add((data_u8[10] as u64).rotate_left(16))
                'r10: 10 => h[3] = h[3].wrapping_add((data_u8[9] as u64).rotate_left(8))
                'r9: 9 => h[3] = h[3].wrapping_add(data_u8[8] as u64)
                'r8: 8 => {
                    h[2] = h[2].wrapping_add(data[0]);
                    break 'done;
                }
                'r7: 7 => h[2] = h[2].wrapping_add((data_u8[6] as u64).rotate_left(48))
                'r6: 6 => h[2] = h[2].wrapping_add((data_u8[5] as u64).rotate_left(40))
                'r5: 5 => h[2] = h[2].wrapping_add((data_u8[4] as u64).rotate_left(32))
                'r4: 4 => {
                    h[2] = h[2].wrapping_add(data_u32[0] as u64);
                    break 'done;
                }
                'r3: 3 => h[2] = h[2].wrapping_add((data_u8[2] as u64).rotate_left(16))
                'r2: 2 => h[2] = h[2].wrapping_add((data_u8[1] as u64).rotate_left(8))
                'r1: 1 => {
                    h[2] = h[2].wrapping_add(data_u8[0] as u64);
                    break 'done;
                }
                'r0: 0 => {
                    h[2] = h[2].wrapping_add(SC_CONST);
                    h[3] = h[3].wrapping_add(SC_CONST);
                    break 'done;
                }
                'error: _ => unreachable!()
                => 'done:
            }
        }

        Self::short_end(&mut h);
        h[0] as u128 | ((h[1] as u128) << 64)
    }

    fn short_mix(h: &mut [u64; 4]) {
        macro_rules! mix {
            ($($i:literal, $r:literal);* $(;)?) => { $(
                h[($i + 2) % 4] = h[($i + 2) % 4].rotate_left($r);
                h[($i + 2) % 4] = h[($i + 2) % 4].wrapping_add(h[($i + 3) % 4]);
                h[$i] ^= h[($i + 2) % 4];
            )* };
        }
        mix! {
            0, 50; 1, 52; 2, 30; 3, 41;
            0, 54; 1, 48; 2, 38; 3, 37;
            0, 62; 1, 34; 2,  5; 3, 36;
        }
    }

    fn short_end(h: &mut [u64; 4]) {
        macro_rules! mix {
            ($($i:literal, $r:literal);* $(;)?) => { $(
                h[($i + 3) % 4] ^= h[($i + 2) % 4];
                h[($i + 2) % 4] = h[($i + 2) % 4].rotate_left($r);
                h[($i + 3) % 4] = h[($i + 3) % 4].wrapping_add(h[($i + 2) % 4]);
            )* };
        }
        mix! {
            0, 15; 1, 52; 2, 26; 3, 51;
            0, 28; 1,  9; 2, 47; 3, 54;
            0, 32; 1, 25; 2, 63;
        }
    }

    fn end(data: &[u64; SC_NUM_VARS], h: &mut [u64; SC_NUM_VARS]) {
        if V::VERSION == 2 {
            h[0] = h[0].wrapping_add(data[0]);
            h[1] = h[1].wrapping_add(data[1]);
            h[2] = h[2].wrapping_add(data[2]);
            h[3] = h[3].wrapping_add(data[3]);
            h[4] = h[4].wrapping_add(data[4]);
            h[5] = h[5].wrapping_add(data[5]);
            h[6] = h[6].wrapping_add(data[6]);
            h[7] = h[7].wrapping_add(data[7]);
            h[8] = h[8].wrapping_add(data[8]);
            h[9] = h[9].wrapping_add(data[9]);
            h[10] = h[10].wrapping_add(data[10]);
            h[11] = h[11].wrapping_add(data[11]);
        }
        Self::end_partial(h);
        Self::end_partial(h);
        Self::end_partial(h);
    }

    fn end_partial(h: &mut [u64; SC_NUM_VARS]) {
        macro_rules! mix {
            ($($i:literal, $r:literal);* $(;)?) => { $(
                h[($i + 11) % SC_NUM_VARS] = h[($i + 11) % SC_NUM_VARS].wrapping_add(h[($i + 1) % SC_NUM_VARS]);
                h[($i + 2) % SC_NUM_VARS] ^= h[($i + 11) % SC_NUM_VARS];
                h[($i + 1) % SC_NUM_VARS] = h[($i + 1) % SC_NUM_VARS].rotate_left($r);
            )* };
        }
        mix! {
            0, 44; 1, 15; 2, 34; 3, 21;
            4, 38; 5, 33; 6, 10; 7, 13;
            8, 38; 9, 53; 10, 42; 11, 54;
        }
    }
}

impl<V: Version> Default for SpookyV<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Version> Hasher<u32> for SpookyV<V> {
    #[inline]
    fn finish(&self) -> u32 {
        <Self as Hasher<u128>>::finish(self) as u32
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        <Self as Hasher<u128>>::write(self, bytes);
    }
}

impl<V: Version> Hasher<u64> for SpookyV<V> {
    #[inline]
    fn finish(&self) -> u64 {
        <Self as Hasher<u128>>::finish(self) as u64
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        <Self as Hasher<u128>>::write(self, bytes);
    }
}

impl<V: Version> Hasher<u128> for SpookyV<V> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut length = bytes.len();

        union U {
            p8: *mut u8,
            p64: *mut u64,
            i: usize,
        }
        let mut u = U { i: 0 };

        let new_length = length + self.remainder as usize;

        if new_length < SC_BUF_SIZE {
            unsafe {
                bytes.as_ptr().copy_to_nonoverlapping(
                    (self.data.as_mut_ptr() as *mut u8).add(self.remainder as usize),
                    length,
                );
            }
            self.length += length;
            self.remainder = new_length as u8;
            return;
        }

        let mut h = if self.length < SC_BUF_SIZE {
            [
                self.state[0],
                self.state[1],
                SC_CONST,
                self.state[0],
                self.state[1],
                SC_CONST,
                self.state[0],
                self.state[1],
                SC_CONST,
                self.state[0],
                self.state[1],
                SC_CONST,
            ]
        } else {
            self.state
        };
        self.length += length;

        if self.remainder != 0 {
            let prefix: u8 = SC_BUF_SIZE as u8 - self.remainder;
            unsafe {
                bytes.as_ptr().copy_to_nonoverlapping(
                    (self.data.as_mut_ptr() as *mut u8).add(self.remainder as usize),
                    prefix as usize,
                );
            }
            u.p64 = self.data.as_mut_ptr();
            Self::mix(
                unsafe { core::slice::from_raw_parts(u.p64, SC_NUM_VARS) }
                    .try_into()
                    .unwrap(),
                &mut h,
            );
            Self::mix(
                unsafe { core::slice::from_raw_parts(u.p64.add(SC_NUM_VARS), SC_NUM_VARS) }
                    .try_into()
                    .unwrap(),
                &mut h,
            );
            u.p8 = unsafe { bytes.as_ptr().add(prefix as usize) as *mut _ };
            length -= prefix as usize;
        } else {
            u.p8 = bytes.as_ptr() as *mut _;
        }

        let end = unsafe { u.p64.add((length / SC_BLOCK_SIZE) * SC_NUM_VARS) };
        let remainder = (length - (unsafe { (end as *const u8).offset_from(u.p8) as usize })) as u8;
        if unsafe { u.i } & 7 == 0 {
            while unsafe { u.p64 } < end {
                Self::mix(
                    unsafe { core::slice::from_raw_parts(u.p64, SC_NUM_VARS) }
                        .try_into()
                        .unwrap(),
                    &mut h,
                );
                u.p64 = unsafe { u.p64.add(SC_NUM_VARS) };
            }
        } else {
            while unsafe { u.p64 } < end {
                unsafe {
                    u.p8.copy_to_nonoverlapping(self.data.as_mut_ptr() as *mut u8, SC_BLOCK_SIZE);
                }
                u.p64 = unsafe { u.p64.add(SC_NUM_VARS) };
            }
        }

        self.remainder = remainder;
        unsafe {
            (end as *const u8)
                .copy_to_nonoverlapping(self.data.as_mut_ptr() as *mut u8, remainder as usize);
        }

        self.state = h;
    }

    #[inline]
    fn finish(&self) -> u128 {
        if self.length < SC_BUF_SIZE {
            return self.short();
        }

        let mut data = self.data.as_ptr();
        let mut remainder: u8 = self.remainder;

        let mut h = self.state;

        if remainder >= SC_BLOCK_SIZE as u8 {
            Self::mix(
                unsafe { core::slice::from_raw_parts(data, SC_NUM_VARS) }
                    .try_into()
                    .unwrap(),
                &mut h,
            );
            data = unsafe { data.add(SC_NUM_VARS) };
            remainder -= SC_BLOCK_SIZE as u8;
        }

        unsafe {
            (data as *mut u8)
                .add(remainder as usize)
                .write_bytes(0, SC_BLOCK_SIZE - remainder as usize)
        };

        unsafe {
            (data as *mut u8).add(SC_BLOCK_SIZE - 1).write(remainder);
        }

        if V::VERSION == 1 {
            Self::mix(
                unsafe { core::slice::from_raw_parts(data, SC_NUM_VARS) }
                    .try_into()
                    .unwrap(),
                &mut h,
            );
        }

        Self::end(
            unsafe { core::slice::from_raw_parts(data, SC_NUM_VARS) }
                .try_into()
                .unwrap(),
            &mut h,
        );

        h[0] as u128 | ((h[1] as u128) << 64)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::RawBytes;

    use super::*;

    #[test]
    fn v1() {
        const EXPECTED: [u32; 512] = [
            0xa24295ec, 0xfe3a05ce, 0x257fd8ef, 0x3acd5217, 0xfdccf85c, 0xc7b5f143, 0x3b0c3ff0,
            0x5220f13c, 0xa6426724, 0x4d5426b4, 0x43e76b26, 0x051bc437, 0xd8f28a02, 0x23ccc30e,
            0x811d1a2d, 0x039128d4, 0x9cd96a73, 0x216e6a8d, 0x97293fe8, 0xe4fc6d09, 0x1ad34423,
            0x9722d7e4, 0x5a6fdeca, 0x3c94a7e1, 0x81a9a876, 0xae3f7c0e, 0x624b50ee, 0x875e5771,
            0x0095ab74, 0x1a7333fb, 0x056a4221, 0xa38351fa, 0x73f575f1, 0x8fded05b, 0x9097138f,
            0xbd74620c, 0x62d3f5f2, 0x07b78bd0, 0xbafdd81e, 0x0638f2ff, 0x1f6e3aeb, 0xa7786473,
            0x71700e1d, 0x6b4625ab, 0xf02867e1, 0xb2b2408f, 0x9ce21ce5, 0xa62baaaf, 0x26720461,
            0x434813ee, 0x33bc0f14, 0xaaab098a, 0x750af488, 0xc31bf476, 0x9cecbf26, 0x94793cf3,
            0xe1a27584, 0xe80c4880, 0x1299f748, 0x25e55ed2, 0x405e3feb, 0x109e2412, 0x3e55f94f,
            0x59575864, 0x365c869d, 0xc9852e6a, 0x12c30c62, 0x47f5b286, 0xb47e488d, 0xa6667571,
            0x78220d67, 0xa49e30b9, 0x2005ef88, 0xf6d3816d, 0x6926834b, 0xe6116805, 0x694777aa,
            0x464af25b, 0x0e0e2d27, 0x0ea92eae, 0x602c2ca9, 0x1d1d79c5, 0x6364f280, 0x939ee1a4,
            0x3b851bd8, 0x5bb6f19f, 0x80b9ed54, 0x3496a9f1, 0xdf815033, 0x91612339, 0x14c516d6,
            0xa3f0a804, 0x5e78e975, 0xf408bcd9, 0x63d525ed, 0xa1e459c3, 0xfde303af, 0x049fc17f,
            0xe7ed4489, 0xfaeefdb6, 0x2b1b2fa8, 0xc67579a6, 0x5505882e, 0xe3e1c7cb, 0xed53bf30,
            0x9e628351, 0x8fa12113, 0x7500c30f, 0xde1bee00, 0xf1fefe06, 0xdc759c00, 0x4c75e5ab,
            0xf889b069, 0x695bf8ae, 0x47d6600f, 0xd2a84f87, 0xa0ca82a9, 0x8d2b750c, 0xe03d8cd7,
            0x581fea33, 0x969b0460, 0x36c7b7de, 0x74b3fd20, 0x2bb8bde6, 0x13b20dec, 0xa2dcee89,
            0xca36229d, 0x06fdb74e, 0x6d9a982d, 0x02503496, 0xbdb4e0d9, 0xbd1f94cf, 0x6d26f82d,
            0xcf5e41cd, 0x88b67b65, 0x3e1b3ee4, 0xb20e5e53, 0x1d9be438, 0xcef9c692, 0x299bd1b2,
            0xb1279627, 0x210b5f3d, 0x5569bd88, 0x9652ed43, 0x7e8e0f8c, 0xdfa01085, 0xcd6d6343,
            0xb8739826, 0xa52ce9a0, 0xd33ef231, 0x1b4d92c2, 0xabfa116d, 0xcdf47800, 0x3a4eefdc,
            0xd01f3bcf, 0x30a32f46, 0xfb54d851, 0x06a98f67, 0xbdcd0a71, 0x21a00949, 0xfe7049c9,
            0x67ef46d2, 0xa1fabcbc, 0xa4c72db4, 0x4a8a910d, 0x85a890ad, 0xc37e9454, 0xfc3d034a,
            0x6f46cc52, 0x742be7a8, 0xe94ecbc5, 0x5f993659, 0x98270309, 0x8d1adae9, 0xea6e035e,
            0x293d5fae, 0x669955b3, 0x5afe23b5, 0x4c74efbf, 0x98106505, 0xfbe09627, 0x3c00e8df,
            0x5b03975d, 0x78edc83c, 0x117c49c6, 0x66cdfc73, 0xfa55c94f, 0x5bf285fe, 0x2db49b7d,
            0xfbfeb8f0, 0xb7631bab, 0x837849f3, 0xf77f3ae5, 0x6e5db9bc, 0xfdd76f15, 0x545abf92,
            0x8b538102, 0xdd5c9b65, 0xa5adfd55, 0xecbd7bc5, 0x9f99ebdd, 0x67500dcb, 0xf5246d1f,
            0x2b0c061c, 0x927a3747, 0xc77ba267, 0x6da9f855, 0x6240d41a, 0xe9d1701d, 0xc69f0c55,
            0x2c2c37cf, 0x12d82191, 0x47be40d3, 0x165b35cd, 0xb7db42e1, 0x358786e4, 0x84b8fc4e,
            0x92f57c28, 0xf9c8bbd7, 0xab95a33d, 0x11009238, 0xe9770420, 0xd6967e2a, 0x97c1589f,
            0x2ee7e7d3, 0x32cc86da, 0xe47767d1, 0x73e9b61e, 0xd35bac45, 0x835a62bb, 0x5d9217b0,
            0x43f3f0ed, 0x8a97911e, 0x4ec7eb55, 0x4b5a988c, 0xb9056683, 0x45456f97, 0x1669fe44,
            0xafb861b8, 0x8e83a19c, 0x0bab08d6, 0xe6a145a9, 0xc31e5fc2, 0x27621f4c, 0x795692fa,
            0xb5e33ab9, 0x1bc786b6, 0x45d1c106, 0x986531c9, 0x40c9a0ec, 0xff0fdf84, 0xa7359a42,
            0xfd1c2091, 0xf73463d4, 0x51b0d635, 0x1d602fb4, 0xc56b69b7, 0x6909d3f7, 0xa04d68f4,
            0x8d1001a7, 0x8ecace50, 0x21ec4765, 0x3530f6b0, 0x645f3644, 0x9963ef1e, 0x2b3c70d5,
            0xa20c823b, 0x8d26dcae, 0x05214e0c, 0x1993896d, 0x62085a35, 0x7b620b67, 0x1dd85da2,
            0x09ce9b1d, 0xd7873326, 0x063ff730, 0xf4ff3c14, 0x09a49d69, 0x532062ba, 0x03ba7729,
            0xbd9a86cc, 0xe26d02a7, 0x7ccbe5d3, 0x4f662214, 0x8b999a66, 0x3d0b92b4, 0x70b210f0,
            0xf5b8f16f, 0x32146d34, 0x430b92bf, 0x8ab6204c, 0x35e6e1ff, 0xc2f6c2fa, 0xa2df8a1a,
            0x887413ec, 0x7cb7a69f, 0x7ac6dbe6, 0x9102d1cb, 0x8892a590, 0xc804fe3a, 0xdfc4920a,
            0xfc829840, 0x8910d2eb, 0x38a210fd, 0x9d840cc9, 0x7b9c827f, 0x3444ca0c, 0x071735ab,
            0x5e9088e4, 0xc995d60e, 0xbe0bb942, 0x17b089ae, 0x050e1054, 0xcf4324f7, 0x1e3e64dd,
            0x436414bb, 0xc48fc2e3, 0x6b6b83d4, 0x9f6558ac, 0x781b22c5, 0x7147cfe2, 0x3c221b4d,
            0xa5602765, 0x8f01a4f0, 0x2a9f14ae, 0x12158cb8, 0x28177c50, 0x1091a165, 0x39e4e4be,
            0x3e451b7a, 0xd965419c, 0x52053005, 0x0798aa53, 0xe6773e13, 0x1207f671, 0xd2ef998b,
            0xab88a38f, 0xc77a8482, 0xa88fb031, 0x5199e0cd, 0x01b30536, 0x46eeb0ef, 0x814259ff,
            0x9789a8cf, 0x376ec5ac, 0x7087034a, 0x948b6bdd, 0x4281e628, 0x2c848370, 0xd76ce66a,
            0xe9b6959e, 0x24321a8e, 0xdeddd622, 0xb890f960, 0xea26c00a, 0x55e7d8b2, 0xeab67f09,
            0x9227fb08, 0xeebbed06, 0xcac1b0d1, 0xb6412083, 0x05d2b0e7, 0x9037624a, 0xc9702198,
            0x2c8d1a86, 0x3e7d416e, 0xc3f1a39f, 0xf04bdce4, 0xc88cdb61, 0xbdc89587, 0x4d29b63b,
            0x6f24c267, 0x4b529c87, 0x573f5a53, 0xdb3316e9, 0x288eb53b, 0xd2c074bd, 0xef44a99a,
            0x2b404d2d, 0xf6706464, 0xfe824f4c, 0xc3debaf8, 0x12f44f98, 0x03135e76, 0xb4888e7f,
            0xb6b2325d, 0x3a138259, 0x513c83ec, 0x2386d214, 0x94555500, 0xfbd1522d, 0xda2af018,
            0x15b054c0, 0x5ad654e6, 0xb6ed00aa, 0xa2f2180e, 0x5f662825, 0xecd11366, 0x1de5e99d,
            0x07afd2ad, 0xcf457b04, 0xe631e10b, 0x83ae8a21, 0x709f0d59, 0x3e278bf9, 0x246816db,
            0x9f5e8fd3, 0xc5b5b5a2, 0xd54a9d5c, 0x4b6f2856, 0x2eb5a666, 0xfc68bdd4, 0x1ed1a7f8,
            0x98a34b75, 0xc895ada9, 0x2907cc69, 0x87b0b455, 0xddaf96d9, 0xe7da15a6, 0x9298c82a,
            0x72bd5cab, 0x2e2a6ad4, 0x7f4b6bb8, 0x525225fe, 0x985abe90, 0xac1fd6e1, 0xb8340f23,
            0x92985159, 0x7d29501d, 0xe75dc744, 0x687501b4, 0x92077dc3, 0x58281a67, 0xe7e8e9be,
            0xd0e64fd1, 0xb2eb0a30, 0x0e1feccd, 0xc0dc4a9e, 0x5c4aeace, 0x2ca5b93c, 0xee0ec34f,
            0xad78467b, 0x0830e76e, 0x0df63f8b, 0x2c2dfd95, 0x9b41ed31, 0x9ff4cddc, 0x1590c412,
            0x2366fc82, 0x7a83294f, 0x9336c4de, 0x2343823c, 0x5b681096, 0xf320e4c2, 0xc22b70e2,
            0xb5fbfb2a, 0x3ebc2fed, 0x11af07bd, 0x429a08c5, 0x42bee387, 0x58629e33, 0xfb63b486,
            0x52135fbe, 0xf1380e60, 0x6355de87, 0x2f0bb19a, 0x167f63ac, 0x507224cf, 0xf7c99d00,
            0x71646f50, 0x74feb1ca, 0x5f9abfdd, 0x278f7d68, 0x70120cd7, 0x4281b0f2, 0xdc8ebe5c,
            0x36c32163, 0x2da1e884, 0x61877598, 0xbef04402, 0x304db695, 0xfa8e9add, 0x503bac31,
            0x0fe04722, 0xf0d59f47, 0xcdc5c595, 0x918c39dd, 0x0cad8d05, 0x6b3ed1eb, 0x4d43e089,
            0x7ab051f8, 0xdeec371f, 0x0f4816ae, 0xf8a1a240, 0xd15317f6, 0xb8efbf0b, 0xcdd05df8,
            0x4fd5633e, 0x7cf19668, 0x25d8f422, 0x72d156f2, 0x2a778502, 0xda7aefb9, 0x4f4f66e8,
            0x19db6bff, 0x74e468da, 0xa754f358, 0x7339ec50, 0x139006f6, 0xefbd0b91, 0x217e9a73,
            0x939bd79c,
        ];

        let mut buf = [0_u8; EXPECTED.len()];

        for i in 0..EXPECTED.len() {
            buf[i] = (i + 128) as u8;
            let saw: u32 = SpookyDefaultBuildHasherV::<V1>::default().hash_one(RawBytes(&buf[..i]));
            assert_eq!(saw, EXPECTED[i], "wrong value at {i}");
        }
    }

    #[test]
    fn v2() {
        const EXPECTED: [u32; 512] = [
            0x6bf50919, 0x70de1d26, 0xa2b37298, 0x35bc5fbf, 0x8223b279, 0x5bcb315e, 0x53fe88a1,
            0xf9f1a233, 0xee193982, 0x54f86f29, 0xc8772d36, 0x9ed60886, 0x5f23d1da, 0x1ed9f474,
            0xf2ef0c89, 0x83ec01f9, 0xf274736c, 0x7e9ac0df, 0xc7aed250, 0xb1015811, 0xe23470f5,
            0x48ac20c4, 0xe2ab3cd5, 0x608f8363, 0xd0639e68, 0xc4e8e7ab, 0x863c7c5b, 0x4ea63579,
            0x99ae8622, 0x170c658b, 0x149ba493, 0x027bca7c, 0xe5cfc8b6, 0xce01d9d7, 0x11103330,
            0x5d1f5ed4, 0xca720ecb, 0xef408aec, 0x733b90ec, 0x855737a6, 0x9856c65f, 0x647411f7,
            0x50777c74, 0xf0f1a8b7, 0x9d7e55a5, 0xc68dd371, 0xfc1af2cc, 0x75728d0a, 0x390e5fdc,
            0xf389b84c, 0xfb0ccf23, 0xc95bad0e, 0x5b1cb85a, 0x6bdae14f, 0x6deb4626, 0x93047034,
            0x6f3266c6, 0xf529c3bd, 0x396322e7, 0x3777d042, 0x1cd6a5a2, 0x197b402e, 0xc28d0d2b,
            0x09c1afb4, 0x069c8bb7, 0x6f9d4e1e, 0xd2621b5c, 0xea68108d, 0x8660cb8f, 0xd61e6de6,
            0x7fba15c7, 0xaacfaa97, 0xdb381902, 0x4ea22649, 0x5d414a1e, 0xc3fc5984, 0xa0fc9e10,
            0x347dc51c, 0x37545fb6, 0x8c84b26b, 0xf57efa5d, 0x56afaf16, 0xb6e1eb94, 0x9218536a,
            0xe3cc4967, 0xd3275ef4, 0xea63536e, 0x6086e499, 0xaccadce7, 0xb0290d82, 0x4ebfd0d6,
            0x46ccc185, 0x2eeb10d3, 0x474e3c8c, 0x23c84aee, 0x3abae1cb, 0x1499b81a, 0xa2993951,
            0xeed176ad, 0xdfcfe84c, 0xde4a961f, 0x4af13fe6, 0xe0069c42, 0xc14de8f5, 0x6e02ce8f,
            0x90d19f7f, 0xbca4a484, 0xd4efdd63, 0x780fd504, 0xe80310e3, 0x03abbc12, 0x90023849,
            0xd6f6fb84, 0xd6b354c5, 0x5b8575f0, 0x758f14e4, 0x450de862, 0x90704afb, 0x47209a33,
            0xf226b726, 0xf858dab8, 0x7c0d6de9, 0xb05ce777, 0xee5ff2d4, 0x7acb6d5c, 0x2d663f85,
            0x41c72a91, 0x82356bf2, 0x94e948ec, 0xd358d448, 0xeca7814d, 0x78cd7950, 0xd6097277,
            0x97782a5d, 0xf43fc6f4, 0x105f0a38, 0x9e170082, 0x4bfe566b, 0x4371d25f, 0xef25a364,
            0x698eb672, 0x74f850e4, 0x4678ff99, 0x4a290dc6, 0x3918f07c, 0x32c7d9cd, 0x9f28e0af,
            0x0d3c5a86, 0x7bfc8a45, 0xddf0c7e1, 0xdeacb86b, 0x970b3c5c, 0x5e29e199, 0xea28346d,
            0x6b59e71b, 0xf8a8a46a, 0x862f6ce4, 0x3ccb740b, 0x08761e9e, 0xbfa01e5f, 0xf17cfa14,
            0x2dbf99fb, 0x7a0be420, 0x06137517, 0xe020b266, 0xd25bfc61, 0xff10ed00, 0x42e6be8b,
            0x029ef587, 0x683b26e0, 0xb08afc70, 0x7c1fd59e, 0xbaae9a70, 0x98c8c801, 0xb6e35a26,
            0x57083971, 0x90a6a680, 0x1b44169e, 0x1dce237c, 0x518e0a59, 0xccb11358, 0x7b8175fb,
            0xb8fe701a, 0x10d259bb, 0xe806ce10, 0x9212be79, 0x4604ae7b, 0x7fa22a84, 0xe715b13a,
            0x0394c3b2, 0x11efbbae, 0xe13d9e19, 0x77e012bd, 0x2d05114c, 0xaecf2ddd, 0xb2a2b4aa,
            0xb9429546, 0x55dce815, 0xc89138f8, 0x46dcae20, 0x1f6f7162, 0x0c557ebc, 0x5b996932,
            0xafbbe7e2, 0xd2bd5f62, 0xff475b9f, 0x9cec7108, 0xeaddcffb, 0x5d751aef, 0xf68f7bdf,
            0xf3f4e246, 0x00983fcd, 0x00bc82bb, 0xbf5fd3e7, 0xe80c7e2c, 0x187d8b1f, 0xefafb9a7,
            0x8f27a148, 0x5c9606a9, 0xf2d2be3e, 0xe992d13a, 0xe4bcd152, 0xce40b436, 0x63d6a1fc,
            0xdc1455c4, 0x64641e39, 0xd83010c9, 0x2d535ae0, 0x5b748f3e, 0xf9a9146b, 0x80f10294,
            0x2859acd4, 0x5fc846da, 0x56d190e9, 0x82167225, 0x98e4daba, 0xbf7865f3, 0x00da7ae4,
            0x9b7cd126, 0x644172f8, 0xde40c78f, 0xe8803efc, 0xdd331a2b, 0x48485c3c, 0x4ed01ddc,
            0x9c0b2d9e, 0xb1c6e9d7, 0xd797d43c, 0x274101ff, 0x3bf7e127, 0x91ebbc56, 0x7ffeb321,
            0x4d42096f, 0xd6e9456a, 0x0bade318, 0x2f40ee0b, 0x38cebf03, 0x0cbc2e72, 0xbf03e704,
            0x7b3e7a9a, 0x8e985acd, 0x90917617, 0x413895f8, 0xf11dde04, 0xc66f8244, 0xe5648174,
            0x6c420271, 0x2469d463, 0x2540b033, 0xdc788e7b, 0xe4140ded, 0x0990630a, 0xa54abed4,
            0x6e124829, 0xd940155a, 0x1c8836f6, 0x38fda06c, 0x5207ab69, 0xf8be9342, 0x774882a8,
            0x56fc0d7e, 0x53a99d6e, 0x8241f634, 0x9490954d, 0x447130aa, 0x8cc4a81f, 0x0868ec83,
            0xc22c642d, 0x47880140, 0xfbff3bec, 0x0f531f41, 0xf845a667, 0x08c15fb7, 0x1996cd81,
            0x86579103, 0xe21dd863, 0x513d7f97, 0x3984a1f1, 0xdfcdc5f4, 0x97766a5e, 0x37e2b1da,
            0x41441f3f, 0xabd9ddba, 0x23b755a9, 0xda937945, 0x103e650e, 0x3eef7c8f, 0x2760ff8d,
            0x2493a4cd, 0x1d671225, 0x3bf4bd4c, 0xed6e1728, 0xc70e9e30, 0x4e05e529, 0x928d5aa6,
            0x164d0220, 0xb5184306, 0x4bd7efb3, 0x63830f11, 0xf3a1526c, 0xf1545450, 0xd41d5df5,
            0x25a5060d, 0x77b368da, 0x4fe33c7e, 0xeae09021, 0xfdb053c4, 0x2930f18d, 0xd37109ff,
            0x8511a781, 0xc7e7cdd7, 0x6aeabc45, 0xebbeaeaa, 0x9a0c4f11, 0xda252cbb, 0x5b248f41,
            0x5223b5eb, 0xe32ab782, 0x8e6a1c97, 0x11d3f454, 0x3e05bd16, 0x0059001d, 0xce13ac97,
            0xf83b2b4c, 0x71db5c9a, 0xdc8655a6, 0x9e98597b, 0x3fcae0a2, 0x75e63ccd, 0x076c72df,
            0x4754c6ad, 0x26b5627b, 0xd818c697, 0x998d5f3d, 0xe94fc7b2, 0x1f49ad1a, 0xca7ff4ea,
            0x9fe72c05, 0xfbd0cbbf, 0xb0388ceb, 0xb76031e3, 0xd0f53973, 0xfb17907c, 0xa4c4c10f,
            0x9f2d8af9, 0xca0e56b0, 0xb0d9b689, 0xfcbf37a3, 0xfede8f7d, 0xf836511c, 0x744003fc,
            0x89eba576, 0xcfdcf6a6, 0xc2007f52, 0xaaaf683f, 0x62d2f9ca, 0xc996f77f, 0x77a7b5b3,
            0x8ba7d0a4, 0xef6a0819, 0xa0d903c0, 0x01b27431, 0x58fffd4c, 0x4827f45c, 0x44eb5634,
            0xae70edfc, 0x591c740b, 0x478bf338, 0x2f3b513b, 0x67bf518e, 0x6fef4a0c, 0x1e0b6917,
            0x5ac0edc5, 0x2e328498, 0x077de7d5, 0x5726020b, 0x2aeda888, 0x45b637ca, 0xcf60858d,
            0x3dc91ae2, 0x3e6d5294, 0xe6900d39, 0x0f634c71, 0x827a5fa4, 0xc713994b, 0x1c363494,
            0x3d43b615, 0xe5fe7d15, 0xf6ada4f2, 0x472099d5, 0x04360d39, 0x7f2a71d0, 0x88a4f5ff,
            0x2c28fac5, 0x4cd64801, 0xfd78dd33, 0xc9bdd233, 0x21e266cc, 0x9bbf419d, 0xcbf7d81d,
            0x80f15f96, 0x04242657, 0x53fb0f66, 0xded11e46, 0xf2fdba97, 0x8d45c9f1, 0x4eeae802,
            0x17003659, 0xb9db81a7, 0xe734b1b2, 0x9503c54e, 0xb7c77c3e, 0x271dd0ab, 0xd8b906b5,
            0x0d540ec6, 0xf03b86e0, 0x0fdb7d18, 0x95e261af, 0xad9ec04e, 0x381f4a64, 0xfec798d7,
            0x09ea20be, 0x0ef4ca57, 0x1e6195bb, 0xfd0da78b, 0xcea1653b, 0x157d9777, 0xf04af50f,
            0xad7baa23, 0xd181714a, 0x9bbdab78, 0x6c7d1577, 0x645eb1e7, 0xa0648264, 0x35839ca6,
            0x2287ef45, 0x32a64ca3, 0x26111f6f, 0x64814946, 0xb0cddaf1, 0x4351c59e, 0x1b30471c,
            0xb970788a, 0x30e9f597, 0xd7e58df1, 0xc6d2b953, 0xf5f37cf4, 0x3d7c419e, 0xf91ecb2d,
            0x9c87fd5d, 0xb22384ce, 0x8c7ac51c, 0x62c96801, 0x57e54091, 0x964536fe, 0x13d3b189,
            0x4afd1580, 0xeba62239, 0xb82ea667, 0xae18d43a, 0xbef04402, 0x1942534f, 0xc54bf260,
            0x3c8267f5, 0xa1020ddd, 0x112fcc8a, 0xde596266, 0xe91d0856, 0xf300c914, 0xed84478e,
            0x5b65009e, 0x4764da16, 0xaf8e07a2, 0x4088dc2c, 0x9a0cad41, 0x2c3f179b, 0xa67b83f7,
            0xf27eab09, 0xdbe10e28, 0xf04c911f, 0xd1169f87, 0x8e1e4976, 0x17f57744, 0xe4f5a33f,
            0x27c2e04b, 0x0b7523bd, 0x07305776, 0xc6be7503, 0x918fa7c9, 0xaf2e2cd9, 0x82046f8e,
            0xcc1c8250,
        ];

        let mut buf = [0_u8; EXPECTED.len()];

        for i in 0..EXPECTED.len() {
            buf[i] = (i + 128) as u8;
            let saw: u32 = SpookyDefaultBuildHasher::default().hash_one(RawBytes(&buf[..i]));
            assert_eq!(saw, EXPECTED[i], "wrong value at {i}");
        }
    }
}
