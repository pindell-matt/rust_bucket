// Copyright 2016 The Rust_Bucket Project Developers. See the COPYRIGHT file at
// the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This
// file may not be copied, modified, or distributed except according to those
// terms.

extern crate serde_codegen;

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    match fs::create_dir(Path::new(&out_dir).join("sc")) {
        Ok(()) => (),
        Err(ref e) if e.kind() == ErrorKind::AlreadyExists => (),
        Err(e) => panic!("{}", e)
    }

    for &(src, dst) in [("src/data.rs.in", "data.rs"),
                       ("src/sc/mod.rs.in", "sc/mod.rs")].into_iter() {
        let src = Path::new(src);
        let dst = Path::new(&out_dir).join(dst);
        serde_codegen::expand(&src, &dst).unwrap()
    }
}
