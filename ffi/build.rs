use std::{env, fs, path::PathBuf};

use path_slash::PathExt;

fn main() {
    println!("cargo:rerun-if-changed=include/cqdb.h");
    println!("cargo:rerun-if-changed=Config.cmake.in");
    println!("cargo:rerun-if-changed=libcqdb.pc.in");

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::create_dir_all(dst.join("include")).unwrap();
    fs::create_dir_all(dst.join("lib/cqdb/cmake")).unwrap();
    fs::create_dir_all(dst.join("lib/pkgconfig")).unwrap();
    fs::copy("include/cqdb.h", dst.join("include/cqdb.h")).unwrap();
    fs::write(
        dst.join("lib/cqdb/cmake/cqdbConfig.cmake"),
        fs::read_to_string("Config.cmake.in")
            .unwrap()
            .replace("@PROJECT_NAME@", "cqdb")
            .replace(
                "@PROJECT_INCLUDE_DIR@",
                &dst.join("include").to_slash().unwrap(),
            ),
    )
    .unwrap();
    fs::write(
        dst.join("lib/pkgconfig/libcqdb.pc"),
        fs::read_to_string("libcqdb.pc.in")
            .unwrap()
            .replace("@prefix@", dst.to_str().unwrap()),
    )
    .unwrap();

    println!("cargo:root={}", dst.to_str().unwrap());
    println!("cargo:include={}/include", dst.to_str().unwrap());
}
