use std::{
    ffi::{CStr, CString},
    fs,
    io::Cursor,
};

use bstr::ByteSlice;
use cqdb::{CQDB, CQDBWriter, Flag};

#[test]
fn test_cqdb_reader() {
    let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(100, db.num());

    // Forward lookups, strings to integer indentifiers
    for i in 0..db.num() {
        let s = format!("{:08}", i);
        let j = db.to_id(&s).unwrap();
        assert_eq!(i, j);
    }
    assert!(db.to_id("non-existing-key").is_none());

    // Backward lookups: integer identifiers to strings.
    for i in 0..db.num() {
        let value = db.to_str(i).unwrap();
        assert_eq!(value, format!("{:08}", i));
    }
    assert!(db.to_str(db.num() + 100).is_none());

    // CQDB iterator
    for item in &db {
        let (i, value) = item.unwrap();
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
        let key = format!("{:013}", id);
        writer.put(&key, id).unwrap();
    }
    drop(writer);

    let buf = fs::read("tests/output/cqdb-writer-1.cqdb").unwrap();
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(100, db.num());

    // Forward lookups, strings to integer indentifiers
    for i in 0..db.num() {
        let s = format!("{:013}", i);
        let j = db.to_id(&s).unwrap();
        assert_eq!(i, j);
    }

    // Backward lookups: integer identifiers to strings.
    for i in 0..db.num() {
        let value = db.to_str(i).unwrap();
        assert_eq!(value, format!("{:013}", i));
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
        // Forward lookups, strings to integer indentifiers
        for id in 0..100 {
            let key = CString::new(format!("{:08}", id)).unwrap();
            let j = cqdb_sys::cqdb_to_id(db, key.as_ptr());
            assert_eq!(id, j);
        }
        // Backward lookups: integer identifiers to strings.
        for id in 0..100 {
            let ptr = cqdb_sys::cqdb_to_string(db, id);
            assert!(!ptr.is_null());
            let key = CStr::from_ptr(ptr).to_str().unwrap();
            assert_eq!(key, format!("{:08}", id));
        }
        cqdb_sys::cqdb_delete(db);
    }
}

#[test]
fn test_cqdb_sys_read_cqdb_writer_12_bytes() {
    let file = fs::File::create("tests/output/cqdb-writer-12bytes.cqdb").unwrap();
    let mut writer = CQDBWriter::new(file).unwrap();
    for id in 0..100 {
        let key = format!("{:012}", id);
        writer.put(&key, id).unwrap();
    }
    drop(writer);

    let buf = fs::read("tests/output/cqdb-writer-12bytes.cqdb").unwrap();
    unsafe {
        let db = cqdb_sys::cqdb_reader(buf.as_ptr() as _, buf.len());
        assert!(!db.is_null());
        // Forward lookups, strings to integer indentifiers
        for id in 0..100 {
            let key = CString::new(format!("{:012}", id)).unwrap();
            let j = cqdb_sys::cqdb_to_id(db, key.as_ptr());
            assert_eq!(id, j);
        }
        cqdb_sys::cqdb_delete(db);
    }
}

fn build_cqdb(keys: &[(&str, u32)], flag: Flag) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    let mut writer = CQDBWriter::with_flag(&mut buf, flag).unwrap();
    for &(key, id) in keys {
        writer.put(key, id).unwrap();
    }
    drop(writer);
    buf.into_inner()
}

#[test]
fn test_new_buffer_too_small() {
    let buf = vec![0u8; 100];
    let err = CQDB::new(&buf).unwrap_err();
    assert!(
        err.to_string().contains("invalid file format"),
        "expected 'invalid file format', got: {}",
        err
    );
}

#[test]
fn test_new_bad_magic() {
    let mut buf = vec![0u8; 2072];
    buf[0..4].copy_from_slice(b"XXXX");
    buf[12..16].copy_from_slice(&0x62445371u32.to_le_bytes());
    let err = CQDB::new(&buf).unwrap_err();
    assert!(
        err.to_string().contains("magic mismatch"),
        "expected 'magic mismatch', got: {}",
        err
    );
}

#[test]
fn test_new_bad_byte_order() {
    let mut buf = vec![0u8; 2072];
    buf[0..4].copy_from_slice(b"CQDB");
    buf[12..16].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
    let err = CQDB::new(&buf).unwrap_err();
    assert!(
        err.to_string().contains("byte order mismatch"),
        "expected 'byte order mismatch', got: {}",
        err
    );
}

#[test]
fn test_new_table_out_of_bounds() {
    let mut buf = build_cqdb(&[("hello", 0)], Flag::NONE);
    for i in 0..256 {
        let off = 24 + i * 8;
        let table_offset = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        if table_offset > 0 {
            buf[off + 4..off + 8].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
            break;
        }
    }
    let err = CQDB::new(&buf).unwrap_err();
    assert!(
        err.to_string().contains("invalid table data"),
        "expected 'invalid table data', got: {}",
        err
    );
}

#[test]
fn test_new_backward_link_out_of_bounds() {
    let mut buf = build_cqdb(&[("hello", 0)], Flag::NONE);
    buf[16..20].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
    let err = CQDB::new(&buf).unwrap_err();
    assert!(
        err.to_string().contains("out of bounds"),
        "expected 'out of bounds' error, got: {}",
        err
    );
}

#[test]
fn test_oneway_writer_reader() {
    let buf = build_cqdb(&[("alpha", 0), ("beta", 1), ("gamma", 2)], Flag::ONEWAY);
    let db = CQDB::new(&buf).unwrap();

    assert_eq!(db.to_id("alpha"), Some(0));
    assert_eq!(db.to_id("beta"), Some(1));
    assert_eq!(db.to_id("gamma"), Some(2));
    assert_eq!(db.to_id("nonexistent"), None);

    assert_eq!(db.to_str(0), None);
    assert_eq!(db.to_str(1), None);
    assert_eq!(db.to_str(2), None);

    assert_eq!(db.iter().count(), 0);
}

#[test]
fn test_empty_database() {
    let buf = build_cqdb(&[], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(db.num(), 0);
    assert_eq!(db.to_id("anything"), None);
    assert_eq!(db.to_str(0), None);
    assert_eq!(db.iter().count(), 0);
}

#[test]
fn test_large_key_fallback() {
    let large_key = "x".repeat(300);
    let buf = build_cqdb(&[(&large_key, 0), ("small", 1)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();

    assert_eq!(db.to_id(&large_key), Some(0));
    assert_eq!(db.to_id("small"), Some(1));
    assert_eq!(
        db.to_str(0).map(|s| s.to_str().unwrap().to_string()),
        Some(large_key)
    );
    assert_eq!(db.to_str(1).map(|s| s.to_str().unwrap()), Some("small"));
}

#[test]
fn test_single_entry_database() {
    let buf = build_cqdb(&[("only", 0)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(db.to_id("only"), Some(0));
    assert_eq!(db.to_str(0).unwrap(), "only");
    assert_eq!(db.to_id("missing"), None);
    assert_eq!(db.to_str(1), None);
}

#[test]
fn test_to_id_nonexistent_many_entries() {
    let keys: Vec<(String, u32)> = (0..500).map(|i| (format!("key_{}", i), i)).collect();
    let refs: Vec<(&str, u32)> = keys.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let buf = build_cqdb(&refs, Flag::NONE);
    let db = CQDB::new(&buf).unwrap();

    for (key, id) in &keys {
        assert_eq!(db.to_id(key), Some(*id), "missing key: {}", key);
    }
    assert_eq!(db.to_id("definitely_not_here"), None);
    assert_eq!(db.to_id("key_500"), None);
    assert_eq!(db.to_id(""), None);
}

#[test]
fn test_to_str_out_of_range() {
    let buf = build_cqdb(&[("a", 0), ("b", 1)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();

    assert!(db.to_str(0).is_some());
    assert!(db.to_str(1).is_some());
    assert_eq!(db.to_str(2), None);
    assert_eq!(db.to_str(100), None);
    assert_eq!(db.to_str(u32::MAX), None);
}

#[test]
fn test_sparse_ids() {
    let buf = build_cqdb(
        &[("first", 0), ("tenth", 10), ("hundredth", 100)],
        Flag::NONE,
    );
    let db = CQDB::new(&buf).unwrap();

    assert_eq!(db.to_id("first"), Some(0));
    assert_eq!(db.to_id("tenth"), Some(10));
    assert_eq!(db.to_id("hundredth"), Some(100));

    assert_eq!(db.to_str(0).unwrap(), "first");
    assert_eq!(db.to_str(10).unwrap(), "tenth");
    assert_eq!(db.to_str(100).unwrap(), "hundredth");

    assert_eq!(db.to_str(1), None);
    assert_eq!(db.to_str(50), None);
    assert_eq!(db.to_str(99), None);
}

#[test]
fn test_iterator_size_hint() {
    let buf = build_cqdb(&[("a", 0), ("b", 1), ("c", 2)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();

    let iter = db.iter();
    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 0);
    assert!(upper.is_some());
    assert!(upper.unwrap() >= 3);
}

#[test]
fn test_iterator_size_hint_oneway() {
    let buf = build_cqdb(&[("a", 0), ("b", 1)], Flag::ONEWAY);
    let db = CQDB::new(&buf).unwrap();

    let iter = db.iter();
    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(0));
}

#[test]
fn test_iterator_stops_at_gap() {
    let buf = build_cqdb(&[("zero", 0), ("five", 5)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();

    let items: Vec<_> = db.iter().collect();
    assert_eq!(items.len(), 1);
    let (id, val) = items[0].as_ref().unwrap();
    assert_eq!(*id, 0);
    assert_eq!(*val, "zero");
}

#[test]
fn test_into_iterator() {
    let buf = build_cqdb(&[("x", 0), ("y", 1), ("z", 2)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();

    let mut count = 0;
    for item in &db {
        let (id, _val) = item.unwrap();
        assert!(id < 3);
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_debug_cqdb() {
    let buf = build_cqdb(&[("hello", 0)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();
    let debug = format!("{:?}", db);
    assert!(debug.contains("CQDB"));
    assert!(debug.contains("header"));
    assert!(debug.contains("num"));
}

#[test]
fn test_debug_writer() {
    let cursor = Cursor::new(Vec::new());
    let writer = CQDBWriter::new(cursor).unwrap();
    let debug = format!("{:?}", writer);
    assert!(debug.contains("CQDBWriter"));
    assert!(debug.contains("flag"));
}

#[test]
fn test_num_accessor() {
    let buf = build_cqdb(&[("a", 0), ("b", 1), ("c", 2), ("d", 3)], Flag::NONE);
    let db = CQDB::new(&buf).unwrap();
    assert_eq!(db.num(), 4);
}
