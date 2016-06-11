#![feature(custom_derive, plugin, unboxed_closures)]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate serde;

use serde::ser::Serialize;

use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::fmt::Debug;
use std::any::Any;

mod sc; // sc == schema

pub fn create_table<T: Serialize>(table: String, t: T) -> io::Result<()> {
    let serialized = serde_json::to_string(&t).unwrap();

    let db_table = format!("./db/{}", table);
    if Path::new(&db_table).exists() { return Ok(()) };

    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_fmt(format_args!("{}", serialized)));

    Ok(())
}

// let deserialized: sc::Coordinates = serde_json::from_str(&serialized).unwrap();

pub fn read_table(table: &'static str) -> String {
    let file = File::open(table).expect("Table does not exist!");
    let buf  = BufReader::new(file);
    buf.lines().map(|l| l.expect("Table read failure!")).collect()
}

#[test]
fn it_can_take_any_struct() {
    let c = sc::Coordinates {x: 7, y: 90};
    let t_n = "test".to_string();
    create_table(t_n, c);
}
