extern crate rustc_serialize;

use std::fs::File;
use std::io::prelude::*;
use std::io;
use rustc_serialize::json;

pub fn write_file(data: String) -> io::Result<()> {
    let mut buffer = try!(File::create("./foo"));
    try!(buffer.write_fmt(format_args!("{}", data)));
    Ok(())
}

#[derive(RustcEncodable, Debug)]
struct TableData {
    test_string: String,
}

#[test]
fn it_works() {
    let mut x = TableData { test_string: String::new() };
    x.test_string = "test me".to_string();
    let encoded = json::encode(&x).unwrap();
    write_file(encoded);
}
