use std::{ffi::CString, fs};

use cqdb::CQDB;

#[test]
fn test_cqdb_reader() {
    let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(100, db.num());

    // Forward lookups, strings to integer indentifiers
    for i in 0..db.num() {
        let s = format!("{:08}", i);
        let j = db.to_id(&s).unwrap();
        assert_eq!(i as i32, j);
    }

    // Backward lookups: integer identifiers to strings.
    for i in 0..db.num() {
        let value = db.to_str(i as i32).unwrap();
        assert_eq!(value, format!("{:08}", i));
    }
}

#[test]
fn test_cqdb_read_cqdb_sys() {
    let name = CString::new("tests/output/cqdb-sys.cqdb").unwrap();
    let mode = CString::new("wb").unwrap();
    unsafe {
        let fp = libc::fopen(name.as_ptr(), mode.as_ptr());
        assert!(!fp.is_null());
        let writer = cqdb_sys::cqdb_writer(fp, 0);
        assert!(!writer.is_null());
        for i in 0..100 {
            let s = CString::new(format!("{:08}", i)).unwrap();
            assert_eq!(0, cqdb_sys::cqdb_writer_put(writer, s.as_ptr(), i));
        }
        assert_eq!(0, cqdb_sys::cqdb_writer_close(writer));
        libc::fclose(fp);
    }
    let buf = fs::read("tests/output/cqdb-sys.cqdb").unwrap();
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(100, db.num());
}
