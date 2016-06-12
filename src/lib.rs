#![feature(custom_derive, plugin, unboxed_closures)]
#![cfg_attr(test, allow(dead_code, unused_must_use, unused_imports))]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate serde;
extern crate time;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use serde::ser::Serialize;
use std::collections::HashMap;

mod sc; // sc is the user defined schema

// private struct for the db
#[derive(Serialize, Deserialize, Debug)]
struct Data<T: Serialize>{
    table:   String,
    next_id: String,
    records: HashMap<String, T>,
}
// public functions first then private functions

pub fn update_table<T: Serialize>(table: String, t: &T) -> io::Result<()> {
    let     serialized = serde_json::to_string(&create_base_data(table.clone(), t)).unwrap();
    let     db_table   = Path::new("./db").join(table);
    let mut buffer     = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

#[allow(unused_must_use)]
pub fn create_table<T: Serialize>(table: String, t: &T) -> io::Result<()> {
    create_db_dir();

    let serialized = serde_json::to_string(&create_base_data(table.clone(), t)).unwrap();
    let db_table   = Path::new("./db").join(table);

    if db_table.exists() {
        return Ok(())
    }

    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

pub fn read_table<P: AsRef<Path>>(table: P) -> String {
    let db_table = Path::new("./db").join(table);
    let file     = File::open(db_table).expect("Table does not exist!");
    let buf      = BufReader::new(file);
    buf.lines().map(|l| l.expect("Table read failure!")).collect()
}

// private functions and tests

fn create_base_data<T: Serialize>(table: String, t: T) -> Data<T> {
    let mut record = HashMap::new();
    record.insert("0".to_string(), t);

    let d = Data {
        table:   table.clone(),
        next_id: "1".to_string(),
        records: record,
    };

    d // return the Data<T> struct
}

fn create_db_dir() -> io::Result<()>{
    if Path::new("./db").exists() {
        return Ok(())
    }

    match fs::create_dir("db") {
        Err(why) => println!("! {:?}", why.kind()),
        Ok(_)    => {},
    }

    Ok(())
}

#[test]
fn it_can_create_a_table_and_take_any_struct_to_add_data() {
    let a = sc::Coordinates {x: 42, y: 9000};
    let b = sc::Coordinates {x: 32, y: 8765};
    let c = sc::Coordinates {x: 42, y: 9000};

    let ex_1 = "{\"table\":\"test\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":42,\"y\":9000}}}";
    let ex_2 = "{\"table\":\"test\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":32,\"y\":8765}}}";

    create_table("test".to_string(), &a);
    assert_eq!(ex_1, read_table("test".to_string()));

    update_table("test".to_string(), &b);
    assert_eq!(ex_2, read_table("test".to_string()));

    let mut now_time = time::get_time();

    for n in 1..1001 {
        update_table("test".to_string(), &c);
    }

    println!("{:?}", (time::get_time() - now_time));
}
