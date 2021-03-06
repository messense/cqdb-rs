#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
use std::{
    ffi::CStr,
    fs::File,
    io::BufWriter,
    os::raw::{c_char, c_int, c_uint, c_void},
    ptr,
};

use cqdb::{CQDBWriter, Flag, CQDB};
use libc::FILE;

#[macro_use]
mod macros;

/// No flag
pub const CQDB_NONE: c_uint = 0;
/// A reverse lookup array is omitted
pub const CQDB_ONEWAY: c_uint = 1;

/// Success
pub const CQDB_SUCCESS: c_int = 0;
/// Invalid id parameters
pub const CQDB_ERROR_INVALIDID: c_int = -1018;
/// Error in file write operations.
pub const CQDB_ERROR_FILEWRITE: c_int = -1021;
/// String not found
pub const CQDB_ERROR_NOTFOUND: c_int = -1023;

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

ffi_fn! {
    /// Delete the CQDB reader.
    fn cqdb_delete(db: *mut cqdb_t) {
        if !db.is_null() {
            unsafe { Box::from_raw(db as *mut CQDB) };
        }
    }
}

ffi_fn! {
    /// Open a new CQDB reader on a memory block.
    fn cqdb_reader(buffer: *const c_void, size: usize) -> *mut cqdb_t {
        let buf = unsafe { std::slice::from_raw_parts(buffer as *const u8, size) };
        if let Ok(db) = CQDB::new(buf) {
            Box::into_raw(Box::new(db)) as *mut cqdb_t
        } else {
            ptr::null_mut()
        }
    }
}

ffi_fn! {
    /// Get the number of associations in the database.
    fn cqdb_num(db: *mut cqdb_t) -> c_int {
        let db = db as *mut CQDB;
        unsafe {
            (*db).num() as c_int
        }
    }
}

ffi_fn! {
    /// Retrieve the identifier associated with a string.
    ///
    /// Returns the non-negative identifier if successful, negative status code otherwise.
    fn cqdb_to_id(db: *mut cqdb_t, s: *const c_char) -> c_int {
        let db = db as *mut CQDB;
        unsafe {
            let c_str = CStr::from_ptr(s).to_str().unwrap();
            (*db).to_id(c_str).map(|id| id as c_int).unwrap_or(CQDB_ERROR_NOTFOUND)
        }
    }
}

ffi_fn! {
    /// Retrieve the string associated with an identifier.
    ///
    /// Pointer to the string associated with the identifier if successful; otherwise NULL.
    fn cqdb_to_string(db: *mut cqdb_t, id: c_int) -> *const c_char {
        let db = db as *mut CQDB;
        if let Some(s) = unsafe { (*db).to_str(id as u32) } {
            // Safety
            // This is safe because s is borrowed from the original buffer
            s.as_ptr() as *const c_char
        } else {
            ptr::null_mut()
        }
    }
}

ffi_fn! {
    /// Create a new CQDB writer on a seekable stream.
    ///
    /// This function initializes a database on the seekable stream and returns the pointer to a `::cqdb_writer_t` instanceto write the database.
    /// The stream must have the writable and binary flags.
    /// The database creation flag must be zero except when the reverse lookup array is unnecessary;
    /// specifying `::CQDB_ONEWAY` flag will save the storage space for the reverse lookup array.
    /// Once calling this function, one should avoid accessing the seekable stream directly until calling `cqdb_writer_close()`.
    fn cqdb_writer(fp: *mut FILE, flag: c_int) -> *mut cqdb_writer_t {
        unsafe {
            let file = new_file_from_libc(fp);
            let buf_writer = BufWriter::new(file);
            let flag = if flag as c_uint == CQDB_ONEWAY {
                Flag::ONEWAY
            } else {
                Flag::NONE
            };
            let writer = match CQDBWriter::with_flag(buf_writer, flag) {
                Ok(writer) => {
                    let inner = Box::into_raw(Box::new(writer)) as *mut tag_cqdb_writer_inner;
                    Box::into_raw(Box::new(cqdb_writer_t { file: fp, inner }))
                }
                Err(_) => ptr::null_mut(),
            };
            writer
        }
    }
}

#[cfg(unix)]
unsafe fn new_file_from_libc(fp: *mut FILE) -> File {
    use std::os::unix::io::FromRawFd;

    // Safely get the file descriptor associated with FILE by fflush()ing its contents first
    // Reference: https://stackoverflow.com/a/31688641
    libc::fflush(fp);
    // `from_raw_fd` takes the ownship of the file descriptor, use `libc::dup` to get a new fd
    let fd = libc::dup(libc::fileno(fp));
    File::from_raw_fd(fd)
}

#[cfg(windows)]
unsafe fn new_file_from_libc(fp: *mut FILE) -> File {
    use std::os::windows::io::{FromRawHandle, RawHandle};

    // Safely get the file descriptor associated with FILE by fflush()ing its contents first
    // Reference: https://stackoverflow.com/a/31688641
    libc::fflush(fp);
    let fd = libc::dup(libc::fileno(fp));
    let handle = libc::get_osfhandle(fd) as RawHandle;
    // `from_raw_handle` takes the ownship of the file descriptor, use `libc::dup` to get a new fd
    File::from_raw_handle(handle)
}

ffi_fn! {
    /// Close a CQDB writer.
    ///
    /// This function finalizes the database on the stream.
    /// If successful, the data remaining on the memory is flushed to the stream;
    /// the stream position is moved to the end of the chunk.
    /// If an unexpected error occurs, this function tries to rewind the stream position to
    /// the original position when the function `cqdb_writer()` was called.
    fn cqdb_writer_close(dbw: *mut cqdb_writer_t) -> c_int {
        if !dbw.is_null() {
            unsafe {
                let inner = (*dbw).inner as *mut CQDBWriter<BufWriter<File>>;
                // Drop CQDBWriter
                Box::from_raw(inner);
                // Re-sync file position so that ftell works correctly
                // Reference: https://stackoverflow.com/a/31688641
                let offset = libc::lseek(libc::fileno((*dbw).file), 0, libc::SEEK_CUR);
                libc::fseek((*dbw).file, offset, libc::SEEK_SET);
                Box::from_raw(dbw);
            }
        }
        CQDB_SUCCESS
    }
}

ffi_fn! {
    /// Put a string/identifier association to the database.
    ///
    /// This function append a string/identifier association into the database.
    /// Make sure that the string and/or identifier have never been inserted to the database
    /// and that the identifier is a non-negative value.
    fn cqdb_writer_put(
        dbw: *mut cqdb_writer_t,
        s: *const c_char,
        id: c_int
    ) -> c_int {
        if id < 0 {
            return CQDB_ERROR_INVALIDID;
        }
        unsafe {
            let dbw = (*dbw).inner as *mut CQDBWriter<BufWriter<File>>;
            let c_str = CStr::from_ptr(s).to_str().unwrap();
            if let Err(_) = (*dbw).put(c_str, id as u32) {
                return CQDB_ERROR_FILEWRITE;
            }
        }
        CQDB_SUCCESS
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{ffi::CString, fs};

    #[test]
    fn test_cqdb_read_cqdb_ffi() {
        let name = CString::new("../tests/output/cqdb-ffi-1.cqdb").unwrap();
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
            let s = CString::new(format!("{:08}", 10)).unwrap();
            assert_eq!(
                CQDB_ERROR_INVALIDID,
                cqdb_writer_put(writer, s.as_ptr(), -1)
            );
            assert_eq!(0, cqdb_writer_close(writer));
            libc::fclose(fp);
        }
        let buf = std::fs::read("../tests/output/cqdb-ffi-1.cqdb").unwrap();
        let db = CQDB::new(&buf).unwrap();
        assert_eq!(100, db.num());
    }

    #[test]
    fn test_cqdb_ffi_read_cqdb_writer() {
        let file = fs::File::create("../tests/output/cqdb-ffi-2.cqdb").unwrap();
        let mut writer = CQDBWriter::new(file).unwrap();
        for id in 0..100 {
            let key = format!("{:08}", id);
            writer.put(&key, id).unwrap();
        }
        drop(writer);

        let buf = fs::read("../tests/output/cqdb-ffi-2.cqdb").unwrap();
        unsafe {
            let db = cqdb_reader(buf.as_ptr() as _, buf.len());
            assert!(!db.is_null());
            let size = cqdb_num(db);
            // Forward lookups, strings to integer indentifiers
            for id in 0..size {
                let key = CString::new(format!("{:08}", id)).unwrap();
                let j = cqdb_to_id(db, key.as_ptr());
                assert_eq!(id, j);
            }
            let key = CString::new("non-exists-key").unwrap();
            let j = cqdb_to_id(db, key.as_ptr());
            assert!(j < 0);

            // Backward lookups: integer identifiers to strings.
            for id in 0..size {
                let ptr = cqdb_to_string(db, id);
                assert!(!ptr.is_null());
                let key = CStr::from_ptr(ptr).to_str().unwrap();
                assert_eq!(key, format!("{:08}", id));
            }
            let ptr = cqdb_to_string(db, size + 100);
            assert!(ptr.is_null());

            cqdb_delete(db);
        }
    }
}
