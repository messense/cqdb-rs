use std::ffi::CString;

use cqdb::CQDB;

#[test]
fn test_cqdb_open() {
    let db = CQDB::open("tests/fixtures/test.cqdb").unwrap();
    assert_eq!(100, db.num());
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
            let s = CString::new(format!("{:<8}", i)).unwrap();
            assert_eq!(0, cqdb_sys::cqdb_writer_put(writer, s.as_ptr(), i));
        }
        assert_eq!(0, cqdb_sys::cqdb_writer_close(writer));
        libc::fclose(fp);
    }
    let db = CQDB::open("tests/output/cqdb-sys.cqdb").unwrap();
    assert_eq!(100, db.num());
}
