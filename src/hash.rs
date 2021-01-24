use ::jhash::{jhash_final, jhash_mix, JHASH_INITVAL};

#[must_use]
pub fn jhash(mut key: &[u8], length: u32, initval: u32) -> u32 {
    let mut a = JHASH_INITVAL.wrapping_add(length).wrapping_add(initval);
    let mut b = a;
    let mut c = a;

    while key.len() > 12 {
        use std::convert::TryInto;
        a = a.wrapping_add(u32::from_ne_bytes(key[..4].try_into().unwrap()));
        b = b.wrapping_add(u32::from_ne_bytes(key[4..8].try_into().unwrap()));
        c = c.wrapping_add(u32::from_ne_bytes(key[8..12].try_into().unwrap()));
        jhash_mix(&mut a, &mut b, &mut c);
        key = &key[12..];
    }

    if key.is_empty() {
        return c;
    }

    c = c.wrapping_add((*key.get(11).unwrap_or(&0) as u32) << 24);
    c = c.wrapping_add((*key.get(10).unwrap_or(&0) as u32) << 16);
    c = c.wrapping_add((*key.get(9).unwrap_or(&0) as u32) << 8);
    c = c.wrapping_add((*key.get(8).unwrap_or(&0) as u32) << 0);

    b = b.wrapping_add((*key.get(7).unwrap_or(&0) as u32) << 24);
    b = b.wrapping_add((*key.get(6).unwrap_or(&0) as u32) << 16);
    b = b.wrapping_add((*key.get(5).unwrap_or(&0) as u32) << 8);
    b = b.wrapping_add((*key.get(4).unwrap_or(&0) as u32) << 0);

    a = a.wrapping_add((*key.get(3).unwrap_or(&0) as u32) << 24);
    a = a.wrapping_add((*key.get(2).unwrap_or(&0) as u32) << 16);
    a = a.wrapping_add((*key.get(1).unwrap_or(&0) as u32) << 8);
    a = a.wrapping_add((*key.get(0).unwrap_or(&0) as u32) << 0);

    jhash_final(a, b, c)
}
