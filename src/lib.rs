#![feature(custom_derive, plugin, unboxed_closures)]
#![cfg_attr(test, allow(dead_code, unused_must_use, unused_imports))]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate serde;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use serde::ser::Serialize;

mod sc; // sc is the user defined schema

pub fn update_table<T: Serialize>(table: String, t: &T) -> io::Result<()> {
    let     serialized = serde_json::to_string(t).unwrap();
    let     db_table   = Path::new("./db").join(table);
    let mut buffer     = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

#[allow(unused_must_use)]
pub fn create_table<P: AsRef<Path>, T: Serialize>(table: P, t: &T) -> io::Result<()> {
    create_db_dir();

    let serialized = serde_json::to_string(t).unwrap();
    let db_table   = Path::new("./db").join(table);

    if db_table.exists() {
        return Ok(())
    }

    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

fn create_db_dir() -> io::Result<()>{
    if Path::new("./db").exists() {
        return Ok(())
    }

    match fs::create_dir("db") {
        Err(why) => println!("! {:?}", why.kind()),
        Ok(_)    => {},
    };

    Ok(())
}

pub fn read_table<P: AsRef<Path>>(table: P) -> String {
    let db_table = Path::new("./db").join(table);
    let file     = File::open(db_table).expect("Table does not exist!");
    let buf      = BufReader::new(file);
    buf.lines().map(|l| l.expect("Table read failure!")).collect()
}

#[test]
fn it_can_create_a_table_and_take_any_struct_to_add_data() {
    let a = sc::Coordinates {x: 42, y: 9000};
    let b = sc::Coordinates {x: 32, y: 9000};
    let c = sc::Coordinates {x: 42, y: 9000};

    create_table("test".to_string(), &a);
    assert_eq!(serde_json::to_string(&a).unwrap(), read_table("test".to_string()));

    update_table("test".to_string(), &b);
    assert_eq!(serde_json::to_string(&b).unwrap(), read_table("test".to_string()));

    update_table("test".to_string(), &c);
}
