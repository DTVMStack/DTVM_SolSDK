// Copyright (c) the DTVM authors Core Contributors
// Copyright (c) The Smart Intermediate Representation Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate lalrpop;

use std::path::Path;
use std::{fs, io};

/// Copy all files in a folder from `src` to `dst`.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Copy all std lib deps
pub fn copy_libs() -> io::Result<()> {
    // Copy static libs to the target folder.
    copy_dir_all("./lib", "./target/lib")?;
    copy_dir_all("./lib", "./target/debug/lib")?;
    copy_dir_all("./lib", "./target/release/lib")?;
    copy_dir_all("./lib", "./target/llvm-cov-target/release/lib")?;
    Ok(())
}

fn main() {
    use std::process::Command;
    Command::new("make").status().unwrap();

    copy_libs().unwrap();

    let output = Command::new("git")
        .args(["describe", "--tags"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={git_hash}");

    // Static link LLVM libs
    #[cfg(feature = "release")]
    static_link_llvm();

    lalrpop::process_root().unwrap();
}

#[allow(dead_code)]
fn static_link_llvm() {
    use std::process::Command;

    println!("Use Static Link");
    // compile our linker
    let cxxflags = Command::new("llvm-config")
        .args(["--cxxflags"])
        .output()
        .expect("could not execute llvm-config");

    let cxxflags = String::from_utf8(cxxflags.stdout).unwrap();

    let mut build = cc::Build::new();

    build.file("src/yul2ir/linker.cpp").cpp(true);

    if !cfg!(target_os = "windows") {
        build.flag("-Wno-unused-parameter");
    }

    for flag in cxxflags.split_whitespace() {
        build.flag(flag);
    }

    build.compile("liblinker.a");

    // add the llvm linker
    let libdir = Command::new("llvm-config")
        .args(["--libdir"])
        .output()
        .unwrap();
    let libdir = String::from_utf8(libdir.stdout).unwrap();

    println!("cargo:libdir={libdir}");
    for lib in &["lldELF", "lldCommon", "lldWasm"] {
        //  "lldCore", "lldDriver", in llvm-12
        println!("cargo:rustc-link-lib=static={lib}");
    }

    // Add all the symbols were not using, needed by Windows and debug builds
    for lib in &["lldMachO", "lldMinGW", "lldCOFF"] {
        // "lldReaderWriter", "lldYAML",  in llvm-12
        println!("cargo:rustc-link-lib=static={lib}");
    }

    // Make sure we have an 8MiB stack on Windows. Windows defaults to a 1MB
    // stack, which is not big enough for debug builds
    #[cfg(windows)]
    println!("cargo:rustc-link-arg=/STACK:8388608");
}
