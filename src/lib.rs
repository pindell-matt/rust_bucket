#![feature(custom_derive, plugin, unboxed_closures)]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate serde;

use serde::ser::Serialize;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::io::BufReader;
use std::path::Path;

mod sc; // sc is the user defined schema

pub fn add_data_to_table<P: AsRef<Path>, T: Serialize>(table: P, t: &T) -> io::Result<()> {
    let serialized = serde_json::to_string(t).unwrap();

    let db_table = Path::new("./db").join(table);
    if db_table.exists() { return Ok(()) };

    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

pub fn create_table<P: AsRef<Path>, T: Serialize>(table: P, t: &T) -> io::Result<()> {
    let serialized = serde_json::to_string(t).unwrap();

    create_db_dir();

    let db_table = Path::new("./db").join(table);
    if db_table.exists() { return Ok(()) };

    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

// private fn: this creates the db dir if it does not exist yet //////////////////////////////////

#[warn(unused_must_use)]
fn create_db_dir() -> io::Result<()>{
    if Path::new("./db").exists() { return Ok(()) }

    match fs::create_dir("db") {
        Err(why) => println!("! {:?}", why.kind()),
        Ok(_) => {},
    };

    Ok(())
}

// end of private method /////////////////////////////////////////////////////////////////////////

pub fn read_table<P: AsRef<Path>>(table: P) -> String {
    let db_table = Path::new("./db").join(table);
    let file     = File::open(db_table).expect("Table does not exist!");
    let buf      = BufReader::new(file);

    buf.lines().map(|l| l.expect("Table read failure!")).collect()
}

#[test]
fn it_can_create_a_table_and_take_any_struct_to_add_data() {
    /////////// This test will only pass if you never change the Coordinate values ///////////////
    let c     = sc::Coordinates {x: 42, y: 9000};
    let t_n   = "test".to_string();
    let t_n_t = t_n.clone();

    create_table(t_n, &c);

    assert_eq!(serde_json::to_string(&c).unwrap(), read_table(t_n_t))
}
