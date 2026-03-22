// Vendored from jhash crate v0.1.1 (BSD-3-Clause)
// Copyright (c) 2017, ANLAB-KAIST
// Copyright (c) 2017, Keunhong Lee
// https://crates.io/crates/jhash

const JHASH_INITVAL: u32 = 0xdeadbeef;

#[inline(always)]
fn jhash_mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *a = a.wrapping_sub(*c);
    *a ^= c.rotate_left(4);
    *c = c.wrapping_add(*b);

    *b = b.wrapping_sub(*a);
    *b ^= a.rotate_left(6);
    *a = a.wrapping_add(*c);

    *c = c.wrapping_sub(*b);
    *c ^= b.rotate_left(8);
    *b = b.wrapping_add(*a);

    *a = a.wrapping_sub(*c);
    *a ^= c.rotate_left(16);
    *c = c.wrapping_add(*b);

    *b = b.wrapping_sub(*a);
    *b ^= a.rotate_left(19);
    *a = a.wrapping_add(*c);

    *c = c.wrapping_sub(*b);
    *c ^= b.rotate_left(4);
    *b = b.wrapping_add(*a);
}

#[inline(always)]
fn jhash_final(mut a: u32, mut b: u32, mut c: u32) -> u32 {
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(14));

    a ^= c;
    a = a.wrapping_sub(c.rotate_left(11));

    b ^= a;
    b = b.wrapping_sub(a.rotate_left(25));

    c ^= b;
    c = c.wrapping_sub(b.rotate_left(16));

    a ^= c;
    a = a.wrapping_sub(c.rotate_left(4));

    b ^= a;
    b = b.wrapping_sub(a.rotate_left(14));

    c ^= b;
    c = c.wrapping_sub(b.rotate_left(24));
    c
}

/// Jenkins hash function for CQDB.
///
/// `length` is passed separately from `key.len()` because CQDB hashes include
/// a virtual NUL terminator (`length = key.len() + 1`).
#[inline]
#[must_use]
pub fn jhash(mut key: &[u8], mut length: u32, initval: u32) -> u32 {
    let mut a = JHASH_INITVAL.wrapping_add(length).wrapping_add(initval);
    let mut b = a;
    let mut c = a;

    while length > 12 {
        a = a.wrapping_add(u32::from_ne_bytes([key[0], key[1], key[2], key[3]]));
        b = b.wrapping_add(u32::from_ne_bytes([key[4], key[5], key[6], key[7]]));
        c = c.wrapping_add(u32::from_ne_bytes([key[8], key[9], key[10], key[11]]));
        jhash_mix(&mut a, &mut b, &mut c);
        key = &key[12..];
        length -= 12;
    }

    if length == 0 {
        return c;
    }

    // Pad remaining bytes into a 12-byte buffer to avoid per-byte bounds checks.
    // Bytes beyond key.len() are zero, matching the virtual NUL terminator.
    let mut tail = [0u8; 12];
    let n = key.len().min(12);
    tail[..n].copy_from_slice(&key[..n]);

    c = c.wrapping_add((tail[11] as u32) << 24);
    c = c.wrapping_add((tail[10] as u32) << 16);
    c = c.wrapping_add((tail[9] as u32) << 8);
    c = c.wrapping_add(tail[8] as u32);

    b = b.wrapping_add((tail[7] as u32) << 24);
    b = b.wrapping_add((tail[6] as u32) << 16);
    b = b.wrapping_add((tail[5] as u32) << 8);
    b = b.wrapping_add(tail[4] as u32);

    a = a.wrapping_add((tail[3] as u32) << 24);
    a = a.wrapping_add((tail[2] as u32) << 16);
    a = a.wrapping_add((tail[1] as u32) << 8);
    a = a.wrapping_add(tail[0] as u32);

    jhash_final(a, b, c)
}

#[cfg(test)]
mod tests {
    use super::jhash;

    #[test]
    fn test_jhash_multiple_of_12() {
        let s = b"0123456789ab";
        assert_eq!(s.len(), 12);
        let h = jhash(s, s.len() as u32 + 1, 0);
        assert_eq!(h, 2677502765);
    }

    #[test]
    fn test_jhash_multiple_of_12_again() {
        let s = b"0123456789ab0123456789ab";
        assert_eq!(s.len(), 24);
        let h = jhash(s, s.len() as u32 + 1, 0);
        assert_eq!(h, 1248740946);
    }
}
