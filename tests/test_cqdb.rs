use std::{ffi::CString, fs};

use cqdb::{CQDBWriter, CQDB};

#[test]
fn test_cqdb_reader() {
    let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(100, db.num());

    // Forward lookups, strings to integer indentifiers
    for i in 0..db.num() {
        let s = format!("{:08}", i);
        let j = db.to_id(&s).unwrap();
        assert_eq!(i as u32, j);
    }

    // Backward lookups: integer identifiers to strings.
    for i in 0..db.num() {
        let value = db.to_str(i as u32).unwrap();
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

#[test]
fn test_cqdb_writer() {
    let file = fs::File::create("tests/output/cqdb-writer-1.cqdb").unwrap();
    let mut writer = CQDBWriter::new(file).unwrap();
    for id in 0..100 {
        let key = format!("{:08}", id);
        writer.put(&key, id).unwrap();
    }
    drop(writer);

    let buf = fs::read("tests/output/cqdb-writer-1.cqdb").unwrap();
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(100, db.num());

    // Forward lookups, strings to integer indentifiers
    for i in 0..db.num() {
        let s = format!("{:08}", i);
        let j = db.to_id(&s).unwrap();
        assert_eq!(i as u32, j);
    }

    // Backward lookups: integer identifiers to strings.
    for i in 0..db.num() {
        let value = db.to_str(i as u32).unwrap();
        assert_eq!(value, format!("{:08}", i));
    }
}

#[test]
fn test_cqdb_sys_read_cqdb_writer() {
    let file = fs::File::create("tests/output/cqdb-writer-2.cqdb").unwrap();
    let mut writer = CQDBWriter::new(file).unwrap();
    for id in 0..100 {
        let key = format!("{:08}", id);
        writer.put(&key, id).unwrap();
    }
    drop(writer);

    let buf = fs::read("tests/output/cqdb-writer-2.cqdb").unwrap();
    unsafe {
        let db = cqdb_sys::cqdb_reader(buf.as_ptr() as _, buf.len());
        assert!(!db.is_null());
        for id in 0..100 {
            let key = CString::new(format!("{:08}", id)).unwrap();
            let j = cqdb_sys::cqdb_to_id(db, key.as_ptr());
            assert_eq!(id, j);
        }
    }
}
