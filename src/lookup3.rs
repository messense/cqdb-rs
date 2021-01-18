#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub ptr: *const libc::c_void,
    pub i: usize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub ptr: *const libc::c_void,
    pub i: usize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub ptr: *const libc::c_void,
    pub i: usize,
}
/*
-------------------------------------------------------------------------------
mix -- mix 3 32-bit values reversibly.

This is reversible, so any information in (a,b,c) before mix() is
still in (a,b,c) after mix().

If four pairs of (a,b,c) inputs are run through mix(), or through
mix() in reverse, there are at least 32 bits of the output that
are sometimes the same for one pair and different for another pair.
This was tested for:
* pairs that differed by one bit, by two bits, in any combination
  of top bits of (a,b,c), or in any combination of bottom bits of
  (a,b,c).
* "differ" is defined as +, -, ^, or ~^.  For + and -, I transformed
  the output delta to a Gray code (a^(a>>1)) so a string of 1's (as
  is commonly produced by subtraction) look like a single 1-bit
  difference.
* the base values were pseudorandom, all zero but one bit set, or
  all zero plus a counter that starts at zero.

Some k values for my "a-=c; a^=rot(c,k); c+=b;" arrangement that
satisfy this are
    4  6  8 16 19  4
    9 15  3 18 27 15
   14  9  3  7 17  3
Well, "9 15 3 18 27 15" didn't quite get 32 bits diffing
for "differ" defined as + with a one-bit base and a two-bit delta.  I
used http://burtleburtle.net/bob/hash/avalanche.html to choose
the operations, constants, and arrangements of the variables.

This does not achieve avalanche.  There are input bits of (a,b,c)
that fail to affect some output bits of (a,b,c), especially of a.  The
most thoroughly mixed value is c, but it doesn't really even achieve
avalanche in c.

This allows some parallelism.  Read-after-writes are good at doubling
the number of bits affected, so the goal of mixing pulls in the opposite
direction as the goal of parallelism.  I did what I could.  Rotates
seem to cost as much as shifts on every machine I could lay my hands
on, and rotates are much kinder to the top and bottom bits, so I used
rotates.
-------------------------------------------------------------------------------
*/
/*
-------------------------------------------------------------------------------
final -- final mixing of 3 32-bit values (a,b,c) into c

Pairs of (a,b,c) values differing in only a few bits will usually
produce values of c that look totally different.  This was tested for
* pairs that differed by one bit, by two bits, in any combination
  of top bits of (a,b,c), or in any combination of bottom bits of
  (a,b,c).
* "differ" is defined as +, -, ^, or ~^.  For + and -, I transformed
  the output delta to a Gray code (a^(a>>1)) so a string of 1's (as
  is commonly produced by subtraction) look like a single 1-bit
  difference.
* the base values were pseudorandom, all zero but one bit set, or
  all zero plus a counter that starts at zero.

These constants passed:
 14 11 25 16 4 14 24
 12 14 25 16 4 14 24
and these came close:
  4  8 15 26 3 22 24
 10  8 15 26 3 22 24
 11  8 15 26 3 22 24
-------------------------------------------------------------------------------
*/
/*
--------------------------------------------------------------------
 This works on all machines.  To be useful, it requires
 -- that the key be an array of u32's, and
 -- that the length be the number of u32's in the key

 The function hashword() is identical to hashlittle() on little-endian
 machines, and identical to hashbig() on big-endian machines,
 except that the length has to be measured in u32s rather than in
 bytes.  hashlittle() is more complicated than hashword() only because
 hashlittle() has to dance around fitting the key bytes into registers.
--------------------------------------------------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn hashword(mut k: *const u32, mut length: usize, initval: u32) -> u32
/* the previous hash, or an arbitrary value */ {
    let mut a: u32 = 0;
    let mut b: u32 = 0;
    let mut c: u32 = 0;
    /* Set up the internal state */
    c = 0xdeadbeef_u32
        .wrapping_add((length as u32) << 2_i32)
        .wrapping_add(initval);
    b = c;
    a = b;
    /*------------------------------------------------- handle most of the key */
    while length > 3 {
        a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32 as u32;
        b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32 as u32;
        c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32 as u32;
        a = (a as libc::c_uint).wrapping_sub(c) as u32;
        a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
        c = (c as libc::c_uint).wrapping_add(b) as u32;
        b = (b as libc::c_uint).wrapping_sub(a) as u32;
        b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
        a = (a as libc::c_uint).wrapping_add(c) as u32;
        c = (c as libc::c_uint).wrapping_sub(b) as u32;
        c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
        b = (b as libc::c_uint).wrapping_add(a) as u32;
        a = (a as libc::c_uint).wrapping_sub(c) as u32;
        a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
        c = (c as libc::c_uint).wrapping_add(b) as u32;
        b = (b as libc::c_uint).wrapping_sub(a) as u32;
        b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
        a = (a as libc::c_uint).wrapping_add(c) as u32;
        c = (c as libc::c_uint).wrapping_sub(b) as u32;
        c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
        b = (b as libc::c_uint).wrapping_add(a) as u32;
        length = (length as libc::c_ulong).wrapping_sub(3_i32 as libc::c_ulong) as usize;
        k = k.offset(3_i32 as isize)
    }
    let mut current_block_46: u64;
    /*------------------------------------------- handle the last 3 u32's */
    match length {
        3 => {
            /* all the case statements fall through */
            c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32
                as u32;
            current_block_46 = 13434239986635690899;
        }
        2 => {
            current_block_46 = 13434239986635690899;
        }
        1 => {
            current_block_46 = 9870956143140806313;
        }
        0 | _ => {
            current_block_46 = 4567019141635105728;
        }
    }
    match current_block_46 {
        13434239986635690899 => {
            b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32
                as u32;
            current_block_46 = 9870956143140806313;
        }
        _ => {}
    }
    match current_block_46 {
        9870956143140806313 => {
            a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                as u32;
            c ^= b;
            c = (c as libc::c_uint)
                .wrapping_sub(b << 14_i32 | b >> (32_i32 - 14_i32))
                as u32;
            a ^= c;
            a = (a as libc::c_uint)
                .wrapping_sub(c << 11_i32 | c >> (32_i32 - 11_i32))
                as u32;
            b ^= a;
            b = (b as libc::c_uint)
                .wrapping_sub(a << 25_i32 | a >> (32_i32 - 25_i32))
                as u32;
            c ^= b;
            c = (c as libc::c_uint)
                .wrapping_sub(b << 16_i32 | b >> (32_i32 - 16_i32))
                as u32;
            a ^= c;
            a = (a as libc::c_uint)
                .wrapping_sub(c << 4_i32 | c >> (32_i32 - 4_i32))
                as u32;
            b ^= a;
            b = (b as libc::c_uint)
                .wrapping_sub(a << 14_i32 | a >> (32_i32 - 14_i32))
                as u32;
            c ^= b;
            c = (c as libc::c_uint)
                .wrapping_sub(b << 24_i32 | b >> (32_i32 - 24_i32))
                as u32
        }
        _ => {}
    }
    /*------------------------------------------------------ report the result */
    c
}
/*
--------------------------------------------------------------------
hashword2() -- same as hashword(), but take two seeds and return two
32-bit values.  pc and pb must both be nonnull, and *pc and *pb must
both be initialized with seeds.  If you pass in (*pb)==0, the output
(*pc) will be the same as the return value from hashword().
--------------------------------------------------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn hashword2(
    mut k: *const u32,
    mut length: usize,
    pc: *mut u32,
    pb: *mut u32,
)
/* IN: more seed OUT: secondary hash value */
{
    let mut a: u32 = 0;
    let mut b: u32 = 0;
    let mut c: u32 = 0;
    /* Set up the internal state */
    c = 0xdeadbeef_u32
        .wrapping_add((length << 2_i32) as u32)
        .wrapping_add(*pc);
    b = c;
    a = b;
    c = (c as libc::c_uint).wrapping_add(*pb) as u32;
    /*------------------------------------------------- handle most of the key */
    while length > 3 {
        a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32 as u32;
        b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32 as u32;
        c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32 as u32;
        a = (a as libc::c_uint).wrapping_sub(c) as u32;
        a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
        c = (c as libc::c_uint).wrapping_add(b) as u32;
        b = (b as libc::c_uint).wrapping_sub(a) as u32;
        b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
        a = (a as libc::c_uint).wrapping_add(c) as u32;
        c = (c as libc::c_uint).wrapping_sub(b) as u32;
        c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
        b = (b as libc::c_uint).wrapping_add(a) as u32;
        a = (a as libc::c_uint).wrapping_sub(c) as u32;
        a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
        c = (c as libc::c_uint).wrapping_add(b) as u32;
        b = (b as libc::c_uint).wrapping_sub(a) as u32;
        b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
        a = (a as libc::c_uint).wrapping_add(c) as u32;
        c = (c as libc::c_uint).wrapping_sub(b) as u32;
        c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
        b = (b as libc::c_uint).wrapping_add(a) as u32;
        length = (length as libc::c_ulong).wrapping_sub(3_i32 as libc::c_ulong) as usize;
        k = k.offset(3_i32 as isize)
    }
    let mut current_block_47: u64;
    /*------------------------------------------- handle the last 3 u32's */
    match length {
        3 => {
            /* all the case statements fall through */
            c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32
                as u32;
            current_block_47 = 6910227548317251488;
        }
        2 => {
            current_block_47 = 6910227548317251488;
        }
        1 => {
            current_block_47 = 4246957660433412216;
        }
        0 | _ => {
            current_block_47 = 10758786907990354186;
        }
    }
    match current_block_47 {
        6910227548317251488 => {
            b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32
                as u32;
            current_block_47 = 4246957660433412216;
        }
        _ => {}
    }
    match current_block_47 {
        4246957660433412216 => {
            a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                as u32;
            c ^= b;
            c = (c as libc::c_uint)
                .wrapping_sub(b << 14_i32 | b >> (32_i32 - 14_i32))
                as u32;
            a ^= c;
            a = (a as libc::c_uint)
                .wrapping_sub(c << 11_i32 | c >> (32_i32 - 11_i32))
                as u32;
            b ^= a;
            b = (b as libc::c_uint)
                .wrapping_sub(a << 25_i32 | a >> (32_i32 - 25_i32))
                as u32;
            c ^= b;
            c = (c as libc::c_uint)
                .wrapping_sub(b << 16_i32 | b >> (32_i32 - 16_i32))
                as u32;
            a ^= c;
            a = (a as libc::c_uint)
                .wrapping_sub(c << 4_i32 | c >> (32_i32 - 4_i32))
                as u32;
            b ^= a;
            b = (b as libc::c_uint)
                .wrapping_sub(a << 14_i32 | a >> (32_i32 - 14_i32))
                as u32;
            c ^= b;
            c = (c as libc::c_uint)
                .wrapping_sub(b << 24_i32 | b >> (32_i32 - 24_i32))
                as u32
        }
        _ => {}
    }
    /*------------------------------------------------------ report the result */
    *pc = c;
    *pb = b;
}
/*
-------------------------------------------------------------------------------
hashlittle() -- hash a variable-length key into a 32-bit value
  k       : the key (the unaligned variable-length array of bytes)
  length  : the length of the key, counting by bytes
  initval : can be any 4-byte value
Returns a 32-bit value.  Every bit of the key affects every bit of
the return value.  Two keys differing by one or two bits will have
totally different hash values.

The best hash table sizes are powers of 2.  There is no need to do
mod a prime (mod is sooo slow!).  If you need less than 32 bits,
use a bitmask.  For example, if you need only 10 bits, do
  h = (h & hashmask(10));
In which case, the hash table should have hashsize(10) elements.

If you are hashing n strings (u8 **)k, do it like this:
  for (i=0, h=0; i<n; ++i) h = hashlittle( k[i], len[i], h);

By Bob Jenkins, 2006.  bob_jenkins@burtleburtle.net.  You may use this
code any way you wish, private, educational, or commercial.  It's free.

Use for hash table lookup, or anything where one collision in 2^^32 is
acceptable.  Do NOT use for cryptographic purposes.
-------------------------------------------------------------------------------
*/
#[no_mangle]
pub unsafe extern "C" fn hashlittle(
    key: *const libc::c_void,
    mut length: usize,
    initval: u32,
) -> u32 {
    let mut a: u32 = 0; /* internal state */
    let mut b: u32 = 0; /* needed for Mac Powerbook G4 */
    let mut c: u32 = 0;
    let mut u: C2RustUnnamed = C2RustUnnamed {
        ptr: std::ptr::null::<libc::c_void>(),
    };
    /* Set up the internal state */
    c = 0xdeadbeef_u32
        .wrapping_add(length as u32)
        .wrapping_add(initval); /* read 32-bit chunks */
    b = c;
    a = b;
    u.ptr = key;
    if 0_i32 != 0 && u.i & 0x3 == 0 {
        let mut k: *const u32 = key as *const u32;
        let _k8: *const u8 = std::ptr::null::<u8>();
        /*------ all but last block: aligned reads and affect 32 bits of (a,b,c) */
        while length > 12 {
            a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                as u32;
            b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32
                as u32;
            c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32
                as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k = k.offset(3_i32 as isize)
        }
        /*----------------------------- handle the last (probably partial) block */
        /*
         * "k[2]&0xffffff" actually reads beyond the end of the string, but
         * then masks off the part it's not allowed to read.  Because the
         * string is aligned, the masked-off tail is in the same word as the
         * rest of the string.  Every machine with memory protection I've seen
         * does it on word boundaries, so is OK with this.  But VALGRIND will
         * still catch it and complain.  The masking trick does make the hash
         * noticably faster for short strings (like English words).
         */
        match length {
            12 => {
                c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                /* zero length strings require no mixing */
            }
            11 => {
                c = (c as libc::c_uint).wrapping_add(
                    *k.offset(2_i32 as isize) & 0xffffff_i32 as libc::c_uint,
                ) as u32; /* need to read the key one byte at a time */
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32; /* read 16-bit chunks */
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            10 => {
                c = (c as libc::c_uint).wrapping_add(
                    *k.offset(2_i32 as isize) & 0xffff_i32 as libc::c_uint,
                ) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            9 => {
                c = (c as libc::c_uint).wrapping_add(
                    *k.offset(2_i32 as isize) & 0xff_i32 as libc::c_uint,
                ) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            8 => {
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            7 => {
                b = (b as libc::c_uint).wrapping_add(
                    *k.offset(1_i32 as isize) & 0xffffff_i32 as libc::c_uint,
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            6 => {
                b = (b as libc::c_uint).wrapping_add(
                    *k.offset(1_i32 as isize) & 0xffff_i32 as libc::c_uint,
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            5 => {
                b = (b as libc::c_uint).wrapping_add(
                    *k.offset(1_i32 as isize) & 0xff_i32 as libc::c_uint,
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            4 => a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32,
            3 => {
                a = (a as libc::c_uint).wrapping_add(
                    *k.offset(0_i32 as isize) & 0xffffff_i32 as libc::c_uint,
                ) as u32
            }
            2 => {
                a = (a as libc::c_uint).wrapping_add(
                    *k.offset(0_i32 as isize) & 0xffff_i32 as libc::c_uint,
                ) as u32
            }
            1 => {
                a = (a as libc::c_uint).wrapping_add(
                    *k.offset(0_i32 as isize) & 0xff_i32 as libc::c_uint,
                ) as u32
            }
            0 => return c,
            _ => {}
        }
    } else if 0_i32 != 0 && u.i & 0x1 == 0 {
        let mut k_0: *const u16 = key as *const u16;
        let mut k8_0: *const u8 = std::ptr::null::<u8>();
        while length > 12
        /*--------------- all but last block: aligned reads and different mixing */
        {
            a = (a as libc::c_uint).wrapping_add(
                (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                    (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                ),
            ) as u32;
            b = (b as libc::c_uint).wrapping_add(
                (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                    (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                ),
            ) as u32;
            c = (c as libc::c_uint).wrapping_add(
                (*k_0.offset(4_i32 as isize) as libc::c_uint).wrapping_add(
                    (*k_0.offset(5_i32 as isize) as u32) << 16_i32,
                ),
            ) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k_0 = k_0.offset(6_i32 as isize)
        }
        k8_0 = k_0 as *const u8;
        let current_block_102: u64;
        match length {
            12 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_0.offset(4_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(5_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                current_block_102 = 4983594971376015098;
                /*----------------------------- handle the last (probably partial) block */
                /* zero length requires no mixing */
            }
            11 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k8_0.offset(10_i32 as isize) as u32) << 16_i32,
                ) as u32; /* fall through */
                current_block_102 = 4853259887228079664; /* fall through */
            }
            10 => {
                current_block_102 = 4853259887228079664; /* fall through */
            }
            9 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k8_0.offset(8_i32 as isize) as libc::c_uint)
                    as u32; /* fall through */
                current_block_102 = 6275773953624082445; /* fall through */
            }
            8 => {
                current_block_102 = 6275773953624082445;
            }
            7 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k8_0.offset(6_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_102 = 9336595757136875624;
            }
            6 => {
                current_block_102 = 9336595757136875624;
            }
            5 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k8_0.offset(4_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_102 = 848114783631734443;
            }
            4 => {
                current_block_102 = 848114783631734443;
            }
            3 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k8_0.offset(2_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_102 = 12527991045972860096;
            }
            2 => {
                current_block_102 = 12527991045972860096;
            }
            1 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k8_0.offset(0_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_102 = 4983594971376015098;
            }
            0 => return c,
            _ => {
                current_block_102 = 4983594971376015098;
            }
        }
        match current_block_102 {
            9336595757136875624 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k_0.offset(2_i32 as isize) as libc::c_uint)
                    as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            6275773953624082445 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            4853259887228079664 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k_0.offset(4_i32 as isize) as libc::c_uint)
                    as u32;
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            848114783631734443 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            12527991045972860096 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k_0.offset(0_i32 as isize) as libc::c_uint)
                    as u32
            }
            _ => {}
        }
    } else {
        let mut k_1: *const u8 = key as *const u8;
        /*--------------- all but the last block: affect some 32 bits of (a,b,c) */
        while length > 12 {
            a = (a as libc::c_uint)
                .wrapping_add(*k_1.offset(0_i32 as isize) as libc::c_uint)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_1.offset(1_i32 as isize) as u32) << 8_i32)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_1.offset(2_i32 as isize) as u32) << 16_i32)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_1.offset(3_i32 as isize) as u32) << 24_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add(*k_1.offset(4_i32 as isize) as libc::c_uint)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_1.offset(5_i32 as isize) as u32) << 8_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_1.offset(6_i32 as isize) as u32) << 16_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_1.offset(7_i32 as isize) as u32) << 24_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add(*k_1.offset(8_i32 as isize) as libc::c_uint)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_1.offset(9_i32 as isize) as u32) << 8_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_1.offset(10_i32 as isize) as u32) << 16_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_1.offset(11_i32 as isize) as u32) << 24_i32)
                as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k_1 = k_1.offset(12_i32 as isize)
        }
        let mut current_block_153: u64;
        /*-------------------------------- last block: affect all 32 bits of (c) */
        match length {
            12 => {
                /* all the case statements fall through */
                c = (c as libc::c_uint).wrapping_add(
                    (*k_1.offset(11_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_153 = 3315337111729158105;
            }
            11 => {
                current_block_153 = 3315337111729158105;
            }
            10 => {
                current_block_153 = 5418677336239499507;
            }
            9 => {
                current_block_153 = 7796877030158056141;
            }
            8 => {
                current_block_153 = 9000140654394160520;
            }
            7 => {
                current_block_153 = 16846429559699824015;
            }
            6 => {
                current_block_153 = 11188519093657326844;
            }
            5 => {
                current_block_153 = 16178596392289208333;
            }
            4 => {
                current_block_153 = 7414272476620430068;
            }
            3 => {
                current_block_153 = 11234461503687749102;
            }
            2 => {
                current_block_153 = 13369523527040680999;
            }
            1 => {
                current_block_153 = 15675019461986351858;
            }
            0 => return c,
            _ => {
                current_block_153 = 2103801789718498838;
            }
        }
        match current_block_153 {
            3315337111729158105 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_1.offset(10_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_153 = 5418677336239499507;
            }
            _ => {}
        }
        match current_block_153 {
            5418677336239499507 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_1.offset(9_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_153 = 7796877030158056141;
            }
            _ => {}
        }
        match current_block_153 {
            7796877030158056141 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k_1.offset(8_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_153 = 9000140654394160520;
            }
            _ => {}
        }
        match current_block_153 {
            9000140654394160520 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_1.offset(7_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_153 = 16846429559699824015;
            }
            _ => {}
        }
        match current_block_153 {
            16846429559699824015 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_1.offset(6_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_153 = 11188519093657326844;
            }
            _ => {}
        }
        match current_block_153 {
            11188519093657326844 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_1.offset(5_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_153 = 16178596392289208333;
            }
            _ => {}
        }
        match current_block_153 {
            16178596392289208333 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k_1.offset(4_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_153 = 7414272476620430068;
            }
            _ => {}
        }
        match current_block_153 {
            7414272476620430068 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_1.offset(3_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_153 = 11234461503687749102;
            }
            _ => {}
        }
        match current_block_153 {
            11234461503687749102 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_1.offset(2_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_153 = 13369523527040680999;
            }
            _ => {}
        }
        match current_block_153 {
            13369523527040680999 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_1.offset(1_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_153 = 15675019461986351858;
            }
            _ => {}
        }
        match current_block_153 {
            15675019461986351858 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k_1.offset(0_i32 as isize) as libc::c_uint)
                    as u32
            }
            _ => {}
        }
    }
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 14_i32 | b >> (32_i32 - 14_i32))
        as u32;
    a ^= c;
    a = (a as libc::c_uint)
        .wrapping_sub(c << 11_i32 | c >> (32_i32 - 11_i32))
        as u32;
    b ^= a;
    b = (b as libc::c_uint)
        .wrapping_sub(a << 25_i32 | a >> (32_i32 - 25_i32))
        as u32;
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 16_i32 | b >> (32_i32 - 16_i32))
        as u32;
    a ^= c;
    a = (a as libc::c_uint)
        .wrapping_sub(c << 4_i32 | c >> (32_i32 - 4_i32))
        as u32;
    b ^= a;
    b = (b as libc::c_uint)
        .wrapping_sub(a << 14_i32 | a >> (32_i32 - 14_i32))
        as u32;
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 24_i32 | b >> (32_i32 - 24_i32))
        as u32;
    c
}
/*
 * hashlittle2: return 2 32-bit hash values
 *
 * This is identical to hashlittle(), except it returns two 32-bit hash
 * values instead of just one.  This is good enough for hash table
 * lookup with 2^^64 buckets, or if you want a second hash if you're not
 * happy with the first, or if you want a probably-unique 64-bit ID for
 * the key.  *pc is better mixed than *pb, so use *pc first.  If you want
 * a 64-bit value do something like "*pc + (((uint64_t)*pb)<<32)".
 */
#[no_mangle]
pub unsafe extern "C" fn hashlittle2(
    key: *const libc::c_void,
    mut length: usize,
    pc: *mut u32,
    pb: *mut u32,
)
/* IN: secondary initval, OUT: secondary hash */
{
    let mut a: u32 = 0; /* internal state */
    let mut b: u32 = 0; /* needed for Mac Powerbook G4 */
    let mut c: u32 = 0;
    let mut u: C2RustUnnamed_0 = C2RustUnnamed_0 {
        ptr: std::ptr::null::<libc::c_void>(),
    };
    /* Set up the internal state */
    c = 0xdeadbeef_u32
        .wrapping_add(length as u32)
        .wrapping_add(*pc); /* read 32-bit chunks */
    b = c;
    a = b;
    c = (c as libc::c_uint).wrapping_add(*pb) as u32;
    u.ptr = key;
    if 0_i32 != 0 && u.i & 0x3 == 0 {
        let mut k: *const u32 = key as *const u32;
        let _k8: *const u8 = std::ptr::null::<u8>();
        /*------ all but last block: aligned reads and affect 32 bits of (a,b,c) */
        while length > 12 {
            a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                as u32;
            b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32
                as u32;
            c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32
                as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k = k.offset(3_i32 as isize)
        }
        /*----------------------------- handle the last (probably partial) block */
        /*
         * "k[2]&0xffffff" actually reads beyond the end of the string, but
         * then masks off the part it's not allowed to read.  Because the
         * string is aligned, the masked-off tail is in the same word as the
         * rest of the string.  Every machine with memory protection I've seen
         * does it on word boundaries, so is OK with this.  But VALGRIND will
         * still catch it and complain.  The masking trick does make the hash
         * noticably faster for short strings (like English words).
         */
        match length {
            12 => {
                c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                /* zero length strings require no mixing */
            }
            11 => {
                c = (c as libc::c_uint).wrapping_add(
                    *k.offset(2_i32 as isize) & 0xffffff_i32 as libc::c_uint,
                ) as u32; /* need to read the key one byte at a time */
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32; /* read 16-bit chunks */
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            10 => {
                c = (c as libc::c_uint).wrapping_add(
                    *k.offset(2_i32 as isize) & 0xffff_i32 as libc::c_uint,
                ) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            9 => {
                c = (c as libc::c_uint).wrapping_add(
                    *k.offset(2_i32 as isize) & 0xff_i32 as libc::c_uint,
                ) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            8 => {
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            7 => {
                b = (b as libc::c_uint).wrapping_add(
                    *k.offset(1_i32 as isize) & 0xffffff_i32 as libc::c_uint,
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            6 => {
                b = (b as libc::c_uint).wrapping_add(
                    *k.offset(1_i32 as isize) & 0xffff_i32 as libc::c_uint,
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            5 => {
                b = (b as libc::c_uint).wrapping_add(
                    *k.offset(1_i32 as isize) & 0xff_i32 as libc::c_uint,
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            4 => a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32,
            3 => {
                a = (a as libc::c_uint).wrapping_add(
                    *k.offset(0_i32 as isize) & 0xffffff_i32 as libc::c_uint,
                ) as u32
            }
            2 => {
                a = (a as libc::c_uint).wrapping_add(
                    *k.offset(0_i32 as isize) & 0xffff_i32 as libc::c_uint,
                ) as u32
            }
            1 => {
                a = (a as libc::c_uint).wrapping_add(
                    *k.offset(0_i32 as isize) & 0xff_i32 as libc::c_uint,
                ) as u32
            }
            0 => {
                *pc = c;
                *pb = b;
                return;
            }
            _ => {}
        }
    } else if 0_i32 != 0 && u.i & 0x1 == 0 {
        let mut k_0: *const u16 = key as *const u16;
        let mut k8_0: *const u8 = std::ptr::null::<u8>();
        while length > 12
        /*--------------- all but last block: aligned reads and different mixing */
        {
            a = (a as libc::c_uint).wrapping_add(
                (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                    (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                ),
            ) as u32;
            b = (b as libc::c_uint).wrapping_add(
                (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                    (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                ),
            ) as u32;
            c = (c as libc::c_uint).wrapping_add(
                (*k_0.offset(4_i32 as isize) as libc::c_uint).wrapping_add(
                    (*k_0.offset(5_i32 as isize) as u32) << 16_i32,
                ),
            ) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k_0 = k_0.offset(6_i32 as isize)
        }
        k8_0 = k_0 as *const u8;
        let current_block_107: u64;
        match length {
            12 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_0.offset(4_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(5_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                current_block_107 = 900943123863005455;
                /*----------------------------- handle the last (probably partial) block */
                /* zero length strings require no mixing */
            }
            11 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k8_0.offset(10_i32 as isize) as u32) << 16_i32,
                ) as u32; /* fall through */
                current_block_107 = 9026781924237172511; /* fall through */
            }
            10 => {
                current_block_107 = 9026781924237172511; /* fall through */
            }
            9 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k8_0.offset(8_i32 as isize) as libc::c_uint)
                    as u32; /* fall through */
                current_block_107 = 4632702683991734266; /* fall through */
            }
            8 => {
                current_block_107 = 4632702683991734266;
            }
            7 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k8_0.offset(6_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_107 = 4917107679789484601;
            }
            6 => {
                current_block_107 = 4917107679789484601;
            }
            5 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k8_0.offset(4_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_107 = 18291657569587714112;
            }
            4 => {
                current_block_107 = 18291657569587714112;
            }
            3 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k8_0.offset(2_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_107 = 3893553619240622090;
            }
            2 => {
                current_block_107 = 3893553619240622090;
            }
            1 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k8_0.offset(0_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_107 = 900943123863005455;
            }
            0 => {
                *pc = c;
                *pb = b;
                return;
            }
            _ => {
                current_block_107 = 900943123863005455;
            }
        }
        match current_block_107 {
            4917107679789484601 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k_0.offset(2_i32 as isize) as libc::c_uint)
                    as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            4632702683991734266 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            9026781924237172511 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k_0.offset(4_i32 as isize) as libc::c_uint)
                    as u32;
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(3_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32;
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            18291657569587714112 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as libc::c_uint).wrapping_add(
                        (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                    ),
                ) as u32
            }
            3893553619240622090 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k_0.offset(0_i32 as isize) as libc::c_uint)
                    as u32
            }
            _ => {}
        }
    } else {
        let mut k_1: *const u8 = key as *const u8;
        /*--------------- all but the last block: affect some 32 bits of (a,b,c) */
        while length > 12 {
            a = (a as libc::c_uint)
                .wrapping_add(*k_1.offset(0_i32 as isize) as libc::c_uint)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_1.offset(1_i32 as isize) as u32) << 8_i32)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_1.offset(2_i32 as isize) as u32) << 16_i32)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_1.offset(3_i32 as isize) as u32) << 24_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add(*k_1.offset(4_i32 as isize) as libc::c_uint)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_1.offset(5_i32 as isize) as u32) << 8_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_1.offset(6_i32 as isize) as u32) << 16_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_1.offset(7_i32 as isize) as u32) << 24_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add(*k_1.offset(8_i32 as isize) as libc::c_uint)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_1.offset(9_i32 as isize) as u32) << 8_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_1.offset(10_i32 as isize) as u32) << 16_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_1.offset(11_i32 as isize) as u32) << 24_i32)
                as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k_1 = k_1.offset(12_i32 as isize)
        }
        let mut current_block_160: u64;
        /*-------------------------------- last block: affect all 32 bits of (c) */
        match length {
            12 => {
                /* all the case statements fall through */
                c = (c as libc::c_uint).wrapping_add(
                    (*k_1.offset(11_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_160 = 17789099964501628722;
                /* zero length strings require no mixing */
            }
            11 => {
                current_block_160 = 17789099964501628722;
            }
            10 => {
                current_block_160 = 9520589643232431964;
            }
            9 => {
                current_block_160 = 8770224102498076252;
            }
            8 => {
                current_block_160 = 11868667610303075556;
            }
            7 => {
                current_block_160 = 15997825400551931295;
            }
            6 => {
                current_block_160 = 17636769085122359583;
            }
            5 => {
                current_block_160 = 4729916395257830952;
            }
            4 => {
                current_block_160 = 10489131089047693169;
            }
            3 => {
                current_block_160 = 9437230201039677552;
            }
            2 => {
                current_block_160 = 17109525036494554974;
            }
            1 => {
                current_block_160 = 11954750908340457487;
            }
            0 => {
                *pc = c;
                *pb = b;
                return;
            }
            _ => {
                current_block_160 = 11359721434352816539;
            }
        }
        match current_block_160 {
            17789099964501628722 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_1.offset(10_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_160 = 9520589643232431964;
            }
            _ => {}
        }
        match current_block_160 {
            9520589643232431964 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_1.offset(9_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_160 = 8770224102498076252;
            }
            _ => {}
        }
        match current_block_160 {
            8770224102498076252 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k_1.offset(8_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_160 = 11868667610303075556;
            }
            _ => {}
        }
        match current_block_160 {
            11868667610303075556 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_1.offset(7_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_160 = 15997825400551931295;
            }
            _ => {}
        }
        match current_block_160 {
            15997825400551931295 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_1.offset(6_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_160 = 17636769085122359583;
            }
            _ => {}
        }
        match current_block_160 {
            17636769085122359583 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_1.offset(5_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_160 = 4729916395257830952;
            }
            _ => {}
        }
        match current_block_160 {
            4729916395257830952 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k_1.offset(4_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_160 = 10489131089047693169;
            }
            _ => {}
        }
        match current_block_160 {
            10489131089047693169 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_1.offset(3_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_160 = 9437230201039677552;
            }
            _ => {}
        }
        match current_block_160 {
            9437230201039677552 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_1.offset(2_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_160 = 17109525036494554974;
            }
            _ => {}
        }
        match current_block_160 {
            17109525036494554974 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_1.offset(1_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_160 = 11954750908340457487;
            }
            _ => {}
        }
        match current_block_160 {
            11954750908340457487 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k_1.offset(0_i32 as isize) as libc::c_uint)
                    as u32
            }
            _ => {}
        }
    }
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 14_i32 | b >> (32_i32 - 14_i32))
        as u32;
    a ^= c;
    a = (a as libc::c_uint)
        .wrapping_sub(c << 11_i32 | c >> (32_i32 - 11_i32))
        as u32;
    b ^= a;
    b = (b as libc::c_uint)
        .wrapping_sub(a << 25_i32 | a >> (32_i32 - 25_i32))
        as u32;
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 16_i32 | b >> (32_i32 - 16_i32))
        as u32;
    a ^= c;
    a = (a as libc::c_uint)
        .wrapping_sub(c << 4_i32 | c >> (32_i32 - 4_i32))
        as u32;
    b ^= a;
    b = (b as libc::c_uint)
        .wrapping_sub(a << 14_i32 | a >> (32_i32 - 14_i32))
        as u32;
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 24_i32 | b >> (32_i32 - 24_i32))
        as u32;
    *pc = c;
    *pb = b;
}
/*
 * hashbig():
 * This is the same as hashword() on big-endian machines.  It is different
 * from hashlittle() on all machines.  hashbig() takes advantage of
 * big-endian byte ordering.
 */
#[no_mangle]
pub unsafe extern "C" fn hashbig(
    key: *const libc::c_void,
    mut length: usize,
    initval: u32,
) -> u32 {
    let mut a: u32 = 0; /* to cast key to (usize) happily */
    let mut b: u32 = 0;
    let mut c: u32 = 0;
    let mut u: C2RustUnnamed_1 = C2RustUnnamed_1 {
        ptr: std::ptr::null::<libc::c_void>(),
    };
    /* Set up the internal state */
    c = 0xdeadbeef_u32
        .wrapping_add(length as u32)
        .wrapping_add(initval); /* read 32-bit chunks */
    b = c;
    a = b;
    u.ptr = key;
    if 0_i32 != 0 && u.i & 0x3 == 0 {
        let mut k: *const u32 = key as *const u32;
        let _k8: *const u8 = std::ptr::null::<u8>();
        /*------ all but last block: aligned reads and affect 32 bits of (a,b,c) */
        while length > 12 {
            a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                as u32;
            b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32
                as u32;
            c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32
                as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k = k.offset(3_i32 as isize)
        }
        /*----------------------------- handle the last (probably partial) block */
        /*
         * "k[2]<<8" actually reads beyond the end of the string, but
         * then shifts out the part it's not allowed to read.  Because the
         * string is aligned, the illegal read is in the same word as the
         * rest of the string.  Every machine with memory protection I've seen
         * does it on word boundaries, so is OK with this.  But VALGRIND will
         * still catch it and complain.  The masking trick does make the hash
         * noticably faster for short strings (like English words).
         */
        match length {
            12 => {
                c = (c as libc::c_uint).wrapping_add(*k.offset(2_i32 as isize)) as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
                /* zero length strings require no mixing */
            }
            11 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k.offset(2_i32 as isize) & 0xffffff00_u32)
                    as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            10 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k.offset(2_i32 as isize) & 0xffff0000_u32)
                    as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            9 => {
                c = (c as libc::c_uint)
                    .wrapping_add(*k.offset(2_i32 as isize) & 0xff000000_u32)
                    as u32;
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            8 => {
                b = (b as libc::c_uint).wrapping_add(*k.offset(1_i32 as isize)) as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            7 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k.offset(1_i32 as isize) & 0xffffff00_u32)
                    as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            6 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k.offset(1_i32 as isize) & 0xffff0000_u32)
                    as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            5 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k.offset(1_i32 as isize) & 0xff000000_u32)
                    as u32;
                a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32
            }
            4 => a = (a as libc::c_uint).wrapping_add(*k.offset(0_i32 as isize)) as u32,
            3 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k.offset(0_i32 as isize) & 0xffffff00_u32)
                    as u32
            }
            2 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k.offset(0_i32 as isize) & 0xffff0000_u32)
                    as u32
            }
            1 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k.offset(0_i32 as isize) & 0xff000000_u32)
                    as u32
            }
            0 => return c,
            _ => {}
        }
    } else {
        let mut k_0: *const u8 = key as *const u8;
        /*--------------- all but the last block: affect some 32 bits of (a,b,c) */
        while length > 12 {
            a = (a as libc::c_uint)
                .wrapping_add((*k_0.offset(0_i32 as isize) as u32) << 24_i32)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_0.offset(1_i32 as isize) as u32) << 16_i32)
                as u32;
            a = (a as libc::c_uint)
                .wrapping_add((*k_0.offset(2_i32 as isize) as u32) << 8_i32)
                as u32;
            a = (a as libc::c_uint).wrapping_add(*k_0.offset(3_i32 as isize) as u32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_0.offset(4_i32 as isize) as u32) << 24_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_0.offset(5_i32 as isize) as u32) << 16_i32)
                as u32;
            b = (b as libc::c_uint)
                .wrapping_add((*k_0.offset(6_i32 as isize) as u32) << 8_i32)
                as u32;
            b = (b as libc::c_uint).wrapping_add(*k_0.offset(7_i32 as isize) as u32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_0.offset(8_i32 as isize) as u32) << 24_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_0.offset(9_i32 as isize) as u32) << 16_i32)
                as u32;
            c = (c as libc::c_uint)
                .wrapping_add((*k_0.offset(10_i32 as isize) as u32) << 8_i32)
                as u32;
            c = (c as libc::c_uint).wrapping_add(*k_0.offset(11_i32 as isize) as u32)
                as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 4_i32 | c >> (32_i32 - 4_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 6_i32 | a >> (32_i32 - 6_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 8_i32 | b >> (32_i32 - 8_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            a = (a as libc::c_uint).wrapping_sub(c) as u32;
            a ^= c << 16_i32 | c >> (32_i32 - 16_i32);
            c = (c as libc::c_uint).wrapping_add(b) as u32;
            b = (b as libc::c_uint).wrapping_sub(a) as u32;
            b ^= a << 19_i32 | a >> (32_i32 - 19_i32);
            a = (a as libc::c_uint).wrapping_add(c) as u32;
            c = (c as libc::c_uint).wrapping_sub(b) as u32;
            c ^= b << 4_i32 | b >> (32_i32 - 4_i32);
            b = (b as libc::c_uint).wrapping_add(a) as u32;
            length =
                (length as libc::c_ulong).wrapping_sub(12_i32 as libc::c_ulong) as usize;
            k_0 = k_0.offset(12_i32 as isize)
        }
        let mut current_block_104: u64;
        /*-------------------------------- last block: affect all 32 bits of (c) */
        match length {
            12 => {
                /* all the case statements fall through */
                c = (c as libc::c_uint)
                    .wrapping_add(*k_0.offset(11_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_104 = 13331581089346929243;
            }
            11 => {
                current_block_104 = 13331581089346929243;
            }
            10 => {
                current_block_104 = 1502925665906196206;
            }
            9 => {
                current_block_104 = 6631915049082055027;
            }
            8 => {
                current_block_104 = 6580043694688701937;
            }
            7 => {
                current_block_104 = 17857527908010492922;
            }
            6 => {
                current_block_104 = 16875135817644795235;
            }
            5 => {
                current_block_104 = 10497608430252991967;
            }
            4 => {
                current_block_104 = 3664501850662793693;
            }
            3 => {
                current_block_104 = 3680826663092050670;
            }
            2 => {
                current_block_104 = 18053655495657782313;
            }
            1 => {
                current_block_104 = 5581893539642003875;
            }
            0 => return c,
            _ => {
                current_block_104 = 4804377075063615140;
            }
        }
        match current_block_104 {
            13331581089346929243 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_0.offset(10_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_104 = 1502925665906196206;
            }
            _ => {}
        }
        match current_block_104 {
            1502925665906196206 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_0.offset(9_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_104 = 6631915049082055027;
            }
            _ => {}
        }
        match current_block_104 {
            6631915049082055027 => {
                c = (c as libc::c_uint).wrapping_add(
                    (*k_0.offset(8_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_104 = 6580043694688701937;
            }
            _ => {}
        }
        match current_block_104 {
            6580043694688701937 => {
                b = (b as libc::c_uint)
                    .wrapping_add(*k_0.offset(7_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_104 = 17857527908010492922;
            }
            _ => {}
        }
        match current_block_104 {
            17857527908010492922 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(6_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_104 = 16875135817644795235;
            }
            _ => {}
        }
        match current_block_104 {
            16875135817644795235 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(5_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_104 = 10497608430252991967;
            }
            _ => {}
        }
        match current_block_104 {
            10497608430252991967 => {
                b = (b as libc::c_uint).wrapping_add(
                    (*k_0.offset(4_i32 as isize) as u32) << 24_i32,
                ) as u32;
                current_block_104 = 3664501850662793693;
            }
            _ => {}
        }
        match current_block_104 {
            3664501850662793693 => {
                a = (a as libc::c_uint)
                    .wrapping_add(*k_0.offset(3_i32 as isize) as libc::c_uint)
                    as u32;
                current_block_104 = 3680826663092050670;
            }
            _ => {}
        }
        match current_block_104 {
            3680826663092050670 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(2_i32 as isize) as u32) << 8_i32,
                ) as u32;
                current_block_104 = 18053655495657782313;
            }
            _ => {}
        }
        match current_block_104 {
            18053655495657782313 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(1_i32 as isize) as u32) << 16_i32,
                ) as u32;
                current_block_104 = 5581893539642003875;
            }
            _ => {}
        }
        match current_block_104 {
            5581893539642003875 => {
                a = (a as libc::c_uint).wrapping_add(
                    (*k_0.offset(0_i32 as isize) as u32) << 24_i32,
                ) as u32
            }
            _ => {}
        }
    }
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 14_i32 | b >> (32_i32 - 14_i32))
        as u32;
    a ^= c;
    a = (a as libc::c_uint)
        .wrapping_sub(c << 11_i32 | c >> (32_i32 - 11_i32))
        as u32;
    b ^= a;
    b = (b as libc::c_uint)
        .wrapping_sub(a << 25_i32 | a >> (32_i32 - 25_i32))
        as u32;
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 16_i32 | b >> (32_i32 - 16_i32))
        as u32;
    a ^= c;
    a = (a as libc::c_uint)
        .wrapping_sub(c << 4_i32 | c >> (32_i32 - 4_i32))
        as u32;
    b ^= a;
    b = (b as libc::c_uint)
        .wrapping_sub(a << 14_i32 | a >> (32_i32 - 14_i32))
        as u32;
    c ^= b;
    c = (c as libc::c_uint)
        .wrapping_sub(b << 24_i32 | b >> (32_i32 - 24_i32))
        as u32;
    c
}
/* SELF_TEST */
