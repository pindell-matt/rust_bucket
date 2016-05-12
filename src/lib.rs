extern crate rustc_serialize;

use std::fs::File;
use std::io::prelude::*;
use std::io;
use rustc_serialize::json;
use std::io::BufReader;
use std::path::Path;

pub fn create_table(data: String, table: &'static str) -> io::Result<()> {
    if Path::new(table).exists() { return Ok(()) };
    let mut buffer = try!(File::create(table.to_string()));
    try!(buffer.write_fmt(format_args!("{}", data)));
    Ok(())
}

pub fn read_table(table: &'static str) -> String {
    let file = File::open(table).expect("Table does not exist!");
    let buf  = BufReader::new(file);
    buf.lines().map(|l| l.expect("Table read failure!")).collect()
}

#[derive(RustcEncodable, Debug)]
struct TableData {
    test_string: String,
}

#[test]
fn it_reads_the_table() {
    let mut x     = TableData { test_string: String::new() };
    x.test_string = "test me".to_string();
    let encoded   = json::encode(&x).unwrap();
    create_table(encoded, "./foo");
    let results = read_table("./foo");
    assert_eq!(results, "{\"test_string\":\"test me\"}");
}

#[test]
fn it_wont_overwrite_existing_file() {
    let mut x     = TableData { test_string: String::new() };
    x.test_string = "will show".to_string();
    let encoded   = json::encode(&x).unwrap();
    create_table(encoded, "./bar");
    let results = read_table("./bar");
    assert_eq!(results, "{\"test_string\":\"will show\"}");

    let mut y     = TableData { test_string: String::new() };
    y.test_string = "won't show".to_string();
    let encoded   = json::encode(&y).unwrap();
    create_table(encoded, "./bar");
    let results = read_table("./bar");
    assert_eq!(results, "{\"test_string\":\"will show\"}");
}
