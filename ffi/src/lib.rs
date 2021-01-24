#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
use std::{
    ffi::CStr,
    fs::File,
    mem::ManuallyDrop,
    os::raw::{c_char, c_int, c_uint, c_void},
    ptr,
};

use cqdb::{CQDBWriter, Flag, CQDB};
use libc::FILE;

/// No flag
pub const CQDB_NONE: c_uint = 0;
/// A reverse lookup array is omitted
pub const CQDB_ONEWAY: c_uint = 1;

/// CQDB Reader API
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct tag_cqdb {
    _unused: [u8; 0],
}

pub type cqdb_t = tag_cqdb;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct tag_cqdb_writer_inner {
    _unused: [u8; 0],
}

/// CQDB Writer API
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct tag_cqdb_writer {
    file: *mut FILE,
    inner: *mut tag_cqdb_writer_inner,
}

pub type cqdb_writer_t = tag_cqdb_writer;

/// Delete the CQDB reader.
#[no_mangle]
pub unsafe extern "C" fn cqdb_delete(db: *mut cqdb_t) {
    if !db.is_null() {
        Box::from_raw(db as *mut CQDB);
    }
}

/// Open a new CQDB reader on a memory block.
#[no_mangle]
pub unsafe extern "C" fn cqdb_reader(buffer: *const c_void, size: usize) -> *mut cqdb_t {
    let buf = std::slice::from_raw_parts(buffer as *const u8, size);
    let db = CQDB::new(buf).unwrap();
    Box::into_raw(Box::new(db)) as *mut cqdb_t
}

/// Get the number of associations in the database.
#[no_mangle]
pub unsafe extern "C" fn cqdb_num(db: *mut cqdb_t) -> c_int {
    let db = db as *mut CQDB;
    (*db).num() as c_int
}

/// Retrieve the identifier associated with a string.
#[no_mangle]
pub unsafe extern "C" fn cqdb_to_id(db: *mut cqdb_t, s: *const c_char) -> c_int {
    let db = db as *mut CQDB;
    let c_str = CStr::from_ptr(s).to_str().unwrap();
    (*db).to_id(c_str).unwrap_or(0) as c_int
}

/// Retrieve the string associated with an identifier.
#[no_mangle]
pub unsafe extern "C" fn cqdb_to_string(db: *mut cqdb_t, id: c_int) -> *const c_char {
    let db = db as *mut CQDB;
    if let Some(s) = (*db).to_str(id as u32) {
        // Safety
        // This is safe because s is borrowed from the original buffer
        s.as_ptr() as *const c_char
    } else {
        ptr::null_mut()
    }
}

/// Create a new CQDB writer on a seekable stream.
///
/// This function initializes a database on the seekable stream and returns the pointer to a `::cqdb_writer_t` instanceto write the database.
/// The stream must have the writable and binary flags.
/// The database creation flag must be zero except when the reverse lookup array is unnecessary;
/// specifying `::CQDB_ONEWAY` flag will save the storage space for the reverse lookup array.
/// Once calling this function, one should avoid accessing the seekable stream directly until calling `cqdb_writer_close()`.
#[no_mangle]
pub unsafe extern "C" fn cqdb_writer(fp: *mut FILE, flag: c_int) -> *mut cqdb_writer_t {
    cqdb_writer_impl(fp, flag)
}

#[cfg(unix)]
unsafe fn cqdb_writer_impl(fp: *mut FILE, flag: c_int) -> *mut cqdb_writer_t {
    use std::os::unix::io::FromRawFd;

    let fd = libc::fileno(fp);
    // Avoid drop the File object since it's borrowed
    let mut file = ManuallyDrop::new(File::from_raw_fd(fd));
    let flag = if flag as c_uint == CQDB_ONEWAY {
        Flag::ONEWAY
    } else {
        Flag::NONE
    };
    let writer = CQDBWriter::with_flag(&mut *file, flag).unwrap();
    let inner = Box::into_raw(Box::new(writer)) as *mut tag_cqdb_writer_inner;
    Box::into_raw(Box::new(cqdb_writer_t { file: fp, inner }))
}

#[cfg(windows)]
unsafe fn cqdb_writer_impl(fp: *mut FILE, flag: c_int) -> *mut cqdb_writer_t {
    use std::os::windows::io::{FromRawHandle, RawHandle};

    let fd = libc::fileno(fp);
    let handle = libc::get_osfhandle(fd) as RawHandle;
    // Avoid drop the File object since it's borrowed
    let mut file = ManuallyDrop::new(File::from_raw_handle(handle));
    let flag = if flag as c_uint == CQDB_ONEWAY {
        Flag::ONEWAY
    } else {
        Flag::NONE
    };
    let writer = CQDBWriter::with_flag(&mut *file, flag).unwrap();
    let inner = Box::into_raw(Box::new(writer)) as *mut tag_cqdb_writer_inner;
    Box::into_raw(Box::new(cqdb_writer_t { file: fp, inner }))
}

/// Close a CQDB writer.
///
/// This function finalizes the database on the stream.
/// If successful, the data remaining on the memory is flushed to the stream;
/// the stream position is moved to the end of the chunk.
/// If an unexpected error occurs, this function tries to rewind the stream position to
/// the original position when the function `cqdb_writer()` was called.
#[no_mangle]
pub unsafe extern "C" fn cqdb_writer_close(dbw: *mut cqdb_writer_t) -> c_int {
    if !dbw.is_null() {
        let inner = (*dbw).inner as *mut CQDBWriter<File>;
        // Drop CQDBWriter
        Box::from_raw(inner);
        // Re-sync file position so that ftell works correctly
        let offset = libc::lseek(libc::fileno((*dbw).file), 0, libc::SEEK_CUR);
        libc::fseek((*dbw).file, offset, libc::SEEK_SET);
        Box::from_raw(dbw);
    }
    // FIXME error no
    0
}

/// Put a string/identifier association to the database.
///
/// This function append a string/identifier association into the database.
/// Make sure that the string and/or identifier have never been inserted to the database
/// and that the identifier is a non-negative value.
#[no_mangle]
pub unsafe extern "C" fn cqdb_writer_put(
    dbw: *mut cqdb_writer_t,
    s: *const c_char,
    id: c_int,
) -> c_int {
    let dbw = (*dbw).inner as *mut CQDBWriter<File>;
    let c_str = CStr::from_ptr(s).to_str().unwrap();
    (*dbw).put(c_str, id as u32).unwrap();
    // FIXME error no
    0
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_cqdb_read_cqdb_ffi() {
        let name = CString::new("../tests/output/cqdb-ffi.cqdb").unwrap();
        let mode = CString::new("wb").unwrap();
        unsafe {
            let fp = libc::fopen(name.as_ptr(), mode.as_ptr());
            assert!(!fp.is_null());
            let writer = cqdb_writer(fp, 0);
            assert!(!writer.is_null());
            for i in 0..100 {
                let s = CString::new(format!("{:08}", i)).unwrap();
                assert_eq!(0, cqdb_writer_put(writer, s.as_ptr(), i));
            }
            assert_eq!(0, cqdb_writer_close(writer));
            libc::fclose(fp);
        }
        let buf = std::fs::read("../tests/output/cqdb-ffi.cqdb").unwrap();
        let db = CQDB::new(&buf).unwrap();
        assert_eq!(100, db.num());
    }
}
