use libc::{
    calloc, ferror, free, fseek, ftell, fwrite, memcmp, memset, realloc, strcmp, strlen, strncpy,
    FILE,
};

use crate::lookup3::hashlittle;

pub const CQDB_ERROR_OCCURRED: u32 = 65536;
pub const CQDB_ONEWAY: u32 = 1;
pub const CQDB_NONE: u32 = 0;
pub const CQDB_ERROR_INVALIDID: i32 = -1018;
pub const CQDB_ERROR_FILESEEK: i32 = -1019;
pub const CQDB_ERROR_FILETELL: i32 = -1020;
pub const CQDB_ERROR_FILEWRITE: i32 = -1021;
pub const CQDB_ERROR_OUTOFMEMORY: i32 = -1022;
pub const CQDB_ERROR_NOTFOUND: i32 = -1023;
pub const CQDB_ERROR: i32 = -1024;
pub const CQDB_SUCCESS: i32 = 0;
/* *
 * Writer for a constant quark database.
 */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tag_cqdb_writer {
    pub flag: u32,
    pub fp: *mut FILE,
    pub begin: u32,
    pub cur: u32,
    pub ht: [table_t; 256],
    pub bwd: *mut u32,
    pub bwd_num: u32,
    pub bwd_size: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct table_t {
    pub num: u32,
    pub size: u32,
    pub bucket: *mut bucket_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bucket_t {
    pub hash: u32,
    pub offset: u32,
}
pub type cqdb_writer_t = tag_cqdb_writer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tableref_t {
    pub offset: u32,
    pub num: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct header_t {
    pub chunkid: [i8; 4],
    pub size: u32,
    pub flag: u32,
    pub byteorder: u32,
    pub bwd_size: u32,
    pub bwd_offset: u32,
}
/* *< Number of elements in the backlink array. */
/* *
 * Constant quark database (CQDB).
 */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tag_cqdb {
    pub buffer: *const u8,
    pub size: usize,
    pub header: header_t,
    pub ht: [table_t; 256],
    pub bwd: *mut u32,
    pub num: libc::c_int,
}
pub type cqdb_t = tag_cqdb;
unsafe extern "C" fn write_uint32(wt: *mut cqdb_writer_t, value: u32) -> usize {
    let mut buffer: [u8; 4] = [0; 4];
    buffer[0] = (value & 0xff) as u8;
    buffer[1] = (value >> 8) as u8;
    buffer[2] = (value >> 16) as u8;
    buffer[3] = (value >> 24) as u8;
    fwrite(
        buffer.as_mut_ptr() as *const libc::c_void,
        ::std::mem::size_of::<u8>(),
        4,
        (*wt).fp,
    )
    .wrapping_div(::std::mem::size_of::<u32>())
}
unsafe extern "C" fn write_data(
    wt: *mut cqdb_writer_t,
    data: *const libc::c_void,
    size: usize,
) -> usize {
    fwrite(data, size, 1, (*wt).fp)
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_writer(
    fp: *mut FILE,
    flag: libc::c_int,
) -> *mut cqdb_writer_t {
    let mut i: libc::c_int = 0;
    let mut dbw: *mut cqdb_writer_t =
        calloc(1, ::std::mem::size_of::<cqdb_writer_t>()) as *mut cqdb_writer_t;
    if !dbw.is_null() {
        /* Initialize cqdb_writer_t members. */
        memset(
            dbw as *mut libc::c_void,
            0_i32,
            ::std::mem::size_of::<cqdb_writer_t>(),
        );
        (*dbw).flag = flag as u32;
        (*dbw).fp = fp;
        (*dbw).begin = ftell((*dbw).fp) as u32;
        (*dbw).cur = (0_i32 as libc::c_ulong)
            .wrapping_add(::std::mem::size_of::<header_t>() as libc::c_ulong)
            .wrapping_add((::std::mem::size_of::<tableref_t>() as libc::c_ulong).wrapping_mul(256))
            as u32;
        /* Initialize the hash tables.*/
        i = 0_i32;
        while i < 256_i32 {
            (*dbw).ht[i as usize].bucket = std::ptr::null_mut::<bucket_t>();
            i += 1
        }
        (*dbw).bwd = std::ptr::null_mut::<u32>();
        (*dbw).bwd_num = 0_u32;
        (*dbw).bwd_size = 0_u32;
        /* Move the file pointer to the offset to the first key/data pair. */
        if fseek(
            (*dbw).fp,
            (*dbw).begin.wrapping_add((*dbw).cur) as libc::c_long,
            0,
        ) != 0
        {
            free(dbw as *mut libc::c_void);
            return std::ptr::null_mut::<cqdb_writer_t>();
            /* Seek error. */
        }
    }
    dbw
}
unsafe extern "C" fn cqdb_writer_delete(dbw: *mut cqdb_writer_t) -> libc::c_int {
    let mut i: libc::c_int = 0;
    /* Free allocated memory blocks. */
    i = 0_i32;
    while i < 256_i32 {
        free((*dbw).ht[i as usize].bucket as *mut libc::c_void);
        i += 1
    }
    free((*dbw).bwd as *mut libc::c_void);
    free(dbw as *mut libc::c_void);
    0
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_writer_put(
    mut dbw: *mut cqdb_writer_t,
    str: *const libc::c_char,
    id: libc::c_int,
) -> libc::c_int {
    let mut current_block: u64;
    let mut ret: libc::c_int = 0;
    let key: *const libc::c_void = str as *const libc::c_void;
    let ksize: u32 = strlen(str).wrapping_add(1) as u32;
    /* Compute the hash value and choose a hash table. */
    let hv: u32 = hashlittle(key, ksize as usize, 0);
    let mut ht: *mut table_t =
        &mut *(*dbw).ht.as_mut_ptr().offset(hv.wrapping_rem(256) as isize) as *mut table_t;
    /* Check for non-negative identifier. */
    if id < 0 {
        ret = CQDB_ERROR_INVALIDID
    } else {
        /* Write out the current data. */
        write_uint32(dbw, id as u32);
        write_uint32(dbw, ksize);
        write_data(dbw, key, ksize as usize);
        if ferror((*dbw).fp) != 0 {
            ret = CQDB_ERROR_FILEWRITE
        } else {
            /* Expand the bucket if necessary. */
            if (*ht).size <= (*ht).num {
                (*ht).size = (*ht).size.wrapping_add(1).wrapping_mul(2);
                (*ht).bucket = realloc(
                    (*ht).bucket as *mut libc::c_void,
                    (::std::mem::size_of::<bucket_t>()).wrapping_mul((*ht).size as usize),
                ) as *mut bucket_t;
                if (*ht).bucket.is_null() {
                    ret = CQDB_ERROR_OUTOFMEMORY as libc::c_int;
                    current_block = 2539196295672303199;
                } else {
                    current_block = 8831408221741692167;
                }
            } else {
                current_block = 8831408221741692167;
            }
            match current_block {
                2539196295672303199 => {}
                _ => {
                    /* Set the hash value and current offset position. */
                    (*(*ht).bucket.offset((*ht).num as isize)).hash = hv;
                    (*(*ht).bucket.offset((*ht).num as isize)).offset = (*dbw).cur;
                    (*ht).num = (*ht).num.wrapping_add(1);
                    /* Store the backlink if specified. */
                    if (*dbw).flag & CQDB_ONEWAY as libc::c_int as libc::c_uint == 0 {
                        /* Expand the backlink array if necessary. */
                        if (*dbw).bwd_size <= id as u32 {
                            let mut size: u32 = (*dbw).bwd_size;
                            while size <= id as u32 {
                                size = size.wrapping_add(1).wrapping_mul(2)
                            }
                            (*dbw).bwd = realloc(
                                (*dbw).bwd as *mut libc::c_void,
                                (::std::mem::size_of::<u32>()).wrapping_mul(size as usize),
                            ) as *mut u32;
                            if (*dbw).bwd.is_null() {
                                ret = CQDB_ERROR_OUTOFMEMORY as libc::c_int;
                                current_block = 2539196295672303199;
                            } else {
                                while (*dbw).bwd_size < size {
                                    let fresh0 = (*dbw).bwd_size;
                                    (*dbw).bwd_size = (*dbw).bwd_size.wrapping_add(1);
                                    *(*dbw).bwd.offset(fresh0 as isize) = 0
                                }
                                current_block = 7205609094909031804;
                            }
                        } else {
                            current_block = 7205609094909031804;
                        }
                        match current_block {
                            2539196295672303199 => {}
                            _ => {
                                if (*dbw).bwd_num <= id as u32 {
                                    (*dbw).bwd_num =
                                        (id as u32).wrapping_add(1_i32 as libc::c_uint)
                                }
                                *(*dbw).bwd.offset(id as isize) = (*dbw).cur;
                                current_block = 9853141518545631134;
                            }
                        }
                    } else {
                        current_block = 9853141518545631134;
                    }
                    match current_block {
                        2539196295672303199 => {}
                        _ => {
                            /* Increment the current position. */
                            (*dbw).cur = ((*dbw).cur as libc::c_ulong).wrapping_add(
                                (::std::mem::size_of::<u32>() as libc::c_ulong)
                                    .wrapping_add(::std::mem::size_of::<u32>() as libc::c_ulong)
                                    .wrapping_add(ksize as libc::c_ulong),
                            ) as u32 as u32;
                            return 0;
                        }
                    }
                }
            }
        }
    }
    (*dbw).flag |= CQDB_ERROR_OCCURRED as libc::c_int as libc::c_uint;
    ret
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_writer_close(mut dbw: *mut cqdb_writer_t) -> libc::c_int {
    let current_block: u64;
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut k: libc::c_int = 0;
    let mut ret: libc::c_int = 0;
    let mut offset: libc::c_long = 0;
    let mut header: header_t = header_t {
        chunkid: [0; 4],
        size: 0,
        flag: 0,
        byteorder: 0,
        bwd_size: 0,
        bwd_offset: 0,
    };
    /* If an error have occurred, just free the memory blocks. */
    if (*dbw).flag & CQDB_ERROR_OCCURRED != 0 {
        cqdb_writer_delete(dbw);
        return 0;
    }
    /* Initialize the file header. */
    strncpy(
        header.chunkid.as_mut_ptr() as *mut libc::c_char,
        b"CQDB\x00" as *const u8 as *const libc::c_char,
        4,
    );
    header.flag = 0;
    header.byteorder = 0x62445371;
    header.bwd_offset = 0;
    header.bwd_size = (*dbw).bwd_num;
    /*
       Store the hash tables. At this moment, the file pointer refers to
       the offset succeeding the last key/data pair.
    */
    i = 0;
    loop {
        if i >= 256_i32 as libc::c_uint {
            current_block = 1538046216550696469;
            break;
        }
        let ht: *mut table_t = &mut *(*dbw).ht.as_mut_ptr().offset(i as isize) as *mut table_t;
        /* Do not write empty hash tables. */
        if !(*ht).bucket.is_null() {
            /*
               Actual bucket will have the double size; half elements
               in the bucket are kept empty.
            */
            let n = (*ht).num.wrapping_mul(2);
            /* Allocate the bucket. */
            let dst: *mut bucket_t =
                calloc(n as usize, ::std::mem::size_of::<bucket_t>()) as *mut bucket_t;
            if dst.is_null() {
                ret = CQDB_ERROR_OUTOFMEMORY as libc::c_int;
                current_block = 18438429198922852023;
                break;
            } else {
                /*
                   Put hash elements to the bucket with the open-address method.
                */
                j = 0;
                while j < (*ht).num {
                    let src: *const bucket_t =
                        &mut *(*ht).bucket.offset(j as isize) as *mut bucket_t;
                    let mut k_0: libc::c_int = ((*src).hash >> 8_i32)
                        .wrapping_rem(n as libc::c_uint)
                        as libc::c_int;
                    /* Find a vacant element. */
                    while (*dst.offset(k_0 as isize)).offset != 0 {
                        k_0 = (k_0 + 1) % n as i32
                    }
                    /* Store the hash element. */
                    (*dst.offset(k_0 as isize)).hash = (*src).hash;
                    (*dst.offset(k_0 as isize)).offset = (*src).offset;
                    j = j.wrapping_add(1)
                }
                /* Write the bucket. */
                k = 0;
                while k < n as i32 {
                    write_uint32(dbw, (*dst.offset(k as isize)).hash);
                    write_uint32(dbw, (*dst.offset(k as isize)).offset);
                    k += 1
                }
                /* Free the bucket. */
                free(dst as *mut libc::c_void);
            }
        }
        i = i.wrapping_add(1)
    }
    match current_block {
        1538046216550696469 => {
            /* Write the backlink array if specified. */
            if (*dbw).flag & CQDB_ONEWAY as libc::c_int as libc::c_uint == 0 && 0 < (*dbw).bwd_size
            {
                /* Store the offset to the head of this array. */
                header.bwd_offset = (ftell((*dbw).fp) - (*dbw).begin as libc::c_long) as u32;
                /* Store the contents of the backlink array. */
                i = 0_i32 as u32;
                while i < (*dbw).bwd_num {
                    write_uint32(dbw, *(*dbw).bwd.offset(i as isize));
                    i = i.wrapping_add(1)
                }
            }
            /* Check for an occurrence of a file-related error. */
            if ferror((*dbw).fp) != 0 {
                ret = CQDB_ERROR_FILEWRITE as libc::c_int
            } else {
                /* Store the current position. */
                offset = ftell((*dbw).fp);
                if offset == -1_i32 as libc::c_long {
                    ret = CQDB_ERROR_FILETELL as libc::c_int
                } else {
                    header.size = (offset as u32).wrapping_sub((*dbw).begin);
                    /* Rewind the current position to the beginning. */
                    if fseek((*dbw).fp, (*dbw).begin as libc::c_long, 0_i32)
                        != 0_i32
                    {
                        ret = CQDB_ERROR_FILESEEK as libc::c_int
                    } else {
                        /* Write the file header. */
                        write_data(
                            dbw,
                            header.chunkid.as_mut_ptr() as *const libc::c_void,
                            4_i32 as usize,
                        );
                        write_uint32(dbw, header.size);
                        write_uint32(dbw, header.flag);
                        write_uint32(dbw, header.byteorder);
                        write_uint32(dbw, header.bwd_size);
                        write_uint32(dbw, header.bwd_offset);
                        /*
                           Write references to hash tables. At this moment, dbw->cur points
                           to the offset succeeding the last key/data pair.
                        */
                        i = 0_i32 as u32;
                        while i < 256_i32 as libc::c_uint {
                            /* Offset to the hash table (or zero for non-existent tables). */
                            write_uint32(
                                dbw,
                                if (*dbw).ht[i as usize].num != 0 {
                                    (*dbw).cur
                                } else {
                                    0_i32 as libc::c_uint
                                },
                            );
                            /* Bucket size is double to the number of elements. */
                            write_uint32(
                                dbw,
                                (*dbw).ht[i as usize]
                                    .num
                                    .wrapping_mul(2_i32 as libc::c_uint),
                            );
                            /* Advance the offset counter. */
                            (*dbw).cur = ((*dbw).cur as libc::c_ulong).wrapping_add(
                                ((*dbw).ht[i as usize]
                                    .num
                                    .wrapping_mul(2_i32 as libc::c_uint)
                                    as libc::c_ulong)
                                    .wrapping_mul(
                                        ::std::mem::size_of::<bucket_t>() as libc::c_ulong
                                    ),
                            ) as u32 as u32;
                            i = i.wrapping_add(1)
                        }
                        /* Check an occurrence of a file-related error. */
                        if ferror((*dbw).fp) != 0 {
                            ret = CQDB_ERROR_FILEWRITE as libc::c_int
                        } else if fseek((*dbw).fp, offset, 0_i32) != 0_i32 {
                            ret = CQDB_ERROR_FILESEEK as libc::c_int
                        } else {
                            cqdb_writer_delete(dbw);
                            return ret;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    /* Seek to the last position. */
    /* Seek to the first position. */
    fseek((*dbw).fp, (*dbw).begin as libc::c_long, 0_i32);
    cqdb_writer_delete(dbw);
    ret
}
unsafe extern "C" fn read_uint32(p: *const u8) -> u32 {
    let mut value: u32 = 0;
    value = *p.offset(0_i32 as isize) as u32;
    value |= (*p.offset(1_i32 as isize) as u32) << 8_i32;
    value |= (*p.offset(2_i32 as isize) as u32) << 16_i32;
    value |= (*p.offset(3_i32 as isize) as u32) << 24_i32;
    value
}
unsafe extern "C" fn read_tableref(mut ref_0: *mut tableref_t, mut p: *const u8) -> *const u8 {
    (*ref_0).offset = read_uint32(p);
    p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
    (*ref_0).num = read_uint32(p);
    p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
    p
}
unsafe extern "C" fn read_bucket(mut p: *const u8, num: u32) -> *mut bucket_t {
    let mut i: u32 = 0;
    let bucket: *mut bucket_t =
        calloc(num as usize, ::std::mem::size_of::<bucket_t>()) as *mut bucket_t;
    i = 0_i32 as u32;
    while i < num {
        (*bucket.offset(i as isize)).hash = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        (*bucket.offset(i as isize)).offset = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        i = i.wrapping_add(1)
    }
    bucket
}
unsafe extern "C" fn read_backward_links(mut p: *const u8, num: u32) -> *mut u32 {
    let mut i: u32 = 0;
    let bwd: *mut u32 = calloc(num as usize, ::std::mem::size_of::<u32>()) as *mut u32;
    i = 0_i32 as u32;
    while i < num {
        *bwd.offset(i as isize) = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        i = i.wrapping_add(1)
    }
    bwd
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_reader(
    buffer: *const libc::c_void,
    size: usize,
) -> *mut cqdb_t {
    let mut i: libc::c_int = 0;
    let mut db: *mut cqdb_t = std::ptr::null_mut::<cqdb_t>();
    /* The minimum size of a valid CQDB is OFFSET_DATA. */
    if size
        < 0usize
            .wrapping_add(::std::mem::size_of::<header_t>())
            .wrapping_add((::std::mem::size_of::<tableref_t>()).wrapping_mul(256))
    {
        return std::ptr::null_mut::<cqdb_t>();
    }
    /* Check the file chunkid. */
    if memcmp(
        buffer,
        b"CQDB\x00" as *const u8 as *const libc::c_char as *const libc::c_void,
        4,
    ) != 0_i32
    {
        return std::ptr::null_mut::<cqdb_t>();
    }
    db = calloc(1, ::std::mem::size_of::<cqdb_t>()) as *mut cqdb_t;
    if !db.is_null() {
        let mut p: *const u8 = std::ptr::null::<u8>();
        /* Set memory block and size. */
        (*db).buffer = buffer as *const u8;
        (*db).size = size;
        /* Read the database header. */
        p = (*db).buffer;
        strncpy(
            (*db).header.chunkid.as_mut_ptr() as *mut libc::c_char,
            p as *const libc::c_char,
            4,
        );
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        (*db).header.size = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        (*db).header.flag = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        (*db).header.byteorder = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        (*db).header.bwd_size = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        (*db).header.bwd_offset = read_uint32(p);
        p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
        /* Check the consistency of byte order. */
        if (*db).header.byteorder != 0x62445371_i32 as libc::c_uint {
            free(db as *mut libc::c_void);
            return std::ptr::null_mut::<cqdb_t>();
        }
        /* Check the chunk size. */
        if size < (*db).header.size as usize {
            free(db as *mut libc::c_void);
            return std::ptr::null_mut::<cqdb_t>();
        }
        /* Set pointers to the hash tables. */
        (*db).num = 0_i32; /* Number of records. */
        p = (*db).buffer.offset(
            (0_i32 as libc::c_ulong)
                .wrapping_add(::std::mem::size_of::<header_t>() as libc::c_ulong)
                as isize,
        );
        i = 0_i32;
        while i < 256_i32 {
            let mut ref_0: tableref_t = tableref_t { offset: 0, num: 0 };
            p = read_tableref(&mut ref_0, p);
            if ref_0.offset != 0 {
                /* Set buckets. */
                (*db).ht[i as usize].bucket =
                    read_bucket((*db).buffer.offset(ref_0.offset as isize), ref_0.num);
                (*db).ht[i as usize].num = ref_0.num
            } else {
                /* An empty hash table. */
                (*db).ht[i as usize].bucket = std::ptr::null_mut::<bucket_t>();
                (*db).ht[i as usize].num = 0_i32 as u32
            }
            /* The number of records is the half of the table size.*/
            (*db).num = ((*db).num as libc::c_uint)
                .wrapping_add(ref_0.num.wrapping_div(2_i32 as libc::c_uint))
                as libc::c_int as libc::c_int;
            i += 1
        }
        /* Set the pointer to the backlink array if any. */
        if (*db).header.bwd_offset != 0 {
            (*db).bwd = read_backward_links(
                (*db).buffer.offset((*db).header.bwd_offset as isize),
                (*db).num as u32,
            )
        } else {
            (*db).bwd = std::ptr::null_mut::<u32>()
        }
    }
    db
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_delete(db: *mut cqdb_t) {
    let mut i: libc::c_int = 0;
    if !db.is_null() {
        i = 0_i32;
        while i < 256_i32 {
            free((*db).ht[i as usize].bucket as *mut libc::c_void);
            i += 1
        }
        free((*db).bwd as *mut libc::c_void);
        free(db as *mut libc::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_to_id(
    db: *mut cqdb_t,
    str: *const libc::c_char,
) -> libc::c_int {
    let hv: u32 = hashlittle(
        str as *const libc::c_void,
        strlen(str).wrapping_add(1),
        0_i32 as u32,
    );
    let t: libc::c_int = hv.wrapping_rem(256_i32 as libc::c_uint) as libc::c_int;
    let ht: *mut table_t = &mut *(*db).ht.as_mut_ptr().offset(t as isize) as *mut table_t;
    if (*ht).num != 0 && !(*ht).bucket.is_null() {
        let n: libc::c_int = (*ht).num as libc::c_int;
        let mut k: libc::c_int =
            (hv >> 8_i32).wrapping_rem(n as libc::c_uint) as libc::c_int;
        let mut p: *mut bucket_t = std::ptr::null_mut::<bucket_t>();
        loop {
            p = &mut *(*ht).bucket.offset(k as isize) as *mut bucket_t;
            if (*p).offset == 0 {
                break;
            }
            if (*p).hash == hv {
                let mut value: libc::c_int = 0;
                let mut ksize: u32 = 0;
                let mut q: *const u8 = (*db).buffer.offset((*p).offset as isize);
                value = read_uint32(q) as libc::c_int;
                q = q.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
                ksize = read_uint32(q);
                q = q.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
                if strcmp(str, q as *const libc::c_char) == 0_i32 {
                    return value;
                }
            }
            k = (k + 1_i32) % n
        }
    }
    CQDB_ERROR_NOTFOUND as libc::c_int
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_to_string(
    db: *mut cqdb_t,
    id: libc::c_int,
) -> *const libc::c_char {
    /* Check if the current database supports the backward look-up. */
    if !(*db).bwd.is_null() && (id as u32) < (*db).header.bwd_size {
        let offset: u32 = *(*db).bwd.offset(id as isize); /* Skip key data. */
        if offset != 0 {
            let mut p: *const u8 = (*db).buffer.offset(offset as isize); /* Skip value size. */
            p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
            p = p.offset(::std::mem::size_of::<u32>() as libc::c_ulong as isize);
            return p as *const libc::c_char;
        }
    }
    std::ptr::null::<libc::c_char>()
}
#[no_mangle]
pub unsafe extern "C" fn cqdb_num(db: *mut cqdb_t) -> libc::c_int {
    (*db).num
}
