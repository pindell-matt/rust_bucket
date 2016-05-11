extern crate rustc_serialize;

use std::fs::File;
use std::io::prelude::*;
use std::io;
use rustc_serialize::json;
use std::io::BufReader;

pub fn write_file(data: String, table: &'static str) -> io::Result<()> {
    let mut buffer = try!(File::create(table.to_string()));
    try!(buffer.write_fmt(format_args!("{}", data)));
    Ok(())
}

pub fn read_table(table: &'static str) -> String {
    let file = File::open(table).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines().map(|l| l.expect("Could not parse line")).collect()
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
    write_file(encoded, "./foo");
}

#[test]
fn it_reads_the_table() {
    let mut x = TableData { test_string: String::new() };
    x.test_string = "test me".to_string();
    let encoded = json::encode(&x).unwrap();
    write_file(encoded, "./foo");
    let results  = read_table("./foo");
    assert_eq!(results, "{\"test_string\":\"test me\"}");
}
