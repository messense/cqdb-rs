use std::{ffi::CString, fs, io::BufWriter};

use cqdb::{CQDBWriter, CQDB};
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("reader");
    group.bench_function("cqdb-rs", |b| {
        let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
        b.iter(|| {
            let _db = CQDB::new(&buf).unwrap();
        })
    });
    group.bench_function("cqdb-c", |b| {
        let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
        b.iter(|| {
            let db = unsafe { cqdb_sys::cqdb_reader(buf.as_ptr() as _, buf.len()) };
            assert!(!db.is_null());
        })
    });
    group.finish();

    let mut group = c.benchmark_group("to_id");
    group.bench_function("cqdb-rs", |b| {
        let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
        let db = CQDB::new(&buf).unwrap();
        b.iter(|| {
            for i in 0..db.num() {
                let s = format!("{:08}", i);
                let _j = db.to_id(&s).unwrap();
            }
        })
    });
    group.bench_function("cqdb-c", |b| {
        let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
        let db = unsafe { cqdb_sys::cqdb_reader(buf.as_ptr() as _, buf.len()) };
        assert!(!db.is_null());
        b.iter(|| {
            for id in 0..100 {
                let key = CString::new(format!("{:08}", id)).unwrap();
                let _j = unsafe { cqdb_sys::cqdb_to_id(db, key.as_ptr()) };
            }
        })
    });
    group.finish();

    let mut group = c.benchmark_group("to_string");
    group.bench_function("cqdb-rs", |b| {
        let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
        let db = CQDB::new(&buf).unwrap();
        b.iter(|| {
            for i in 0..db.num() {
                let _value = db.to_str(i as u32).unwrap();
            }
        })
    });
    group.bench_function("cqdb-c", |b| {
        let buf = fs::read("tests/fixtures/test.cqdb").unwrap();
        let db = unsafe { cqdb_sys::cqdb_reader(buf.as_ptr() as _, buf.len()) };
        b.iter(|| {
            assert!(!db.is_null());
            for id in 0..100 {
                let _ptr = unsafe { cqdb_sys::cqdb_to_string(db, id) };
            }
        })
    });
    group.finish();

    let mut group = c.benchmark_group("writer");
    group.bench_function("cqdb-rs", |b| {
        b.iter(|| {
            let file = fs::File::create("tests/output/cqdb-writer-bench-1.cqdb").unwrap();
            let buf_writer = BufWriter::new(file);
            let mut writer = CQDBWriter::new(buf_writer).unwrap();
            for id in 0..100 {
                let key = format!("{:08}", id);
                writer.put(&key, id).unwrap();
            }
            drop(writer);
        });
        fs::remove_file("tests/output/cqdb-writer-bench-1.cqdb").unwrap();
    });
    group.bench_function("cqdb-c", |b| {
        b.iter(|| {
            let name = CString::new("tests/output/cqdb-writer-bench-1.cqdb").unwrap();
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
        });
        fs::remove_file("tests/output/cqdb-writer-bench-1.cqdb").unwrap();
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
