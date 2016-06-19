// Copyright 2016 The Fe_Bucket Project Developers. See the COPYRIGHT file at
// the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This
// file may not be copied, modified, or distributed except according to those
// terms.

#![feature(custom_derive, plugin)]
#![cfg_attr(test, feature(test))]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate serde;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use serde::Deserialize;
use std::io::prelude::*;
use serde::ser::Serialize;
use std::collections::HashMap;

mod sc; // sc is the user defined schema

pub mod errors;
use errors::{Result, Error};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Data<T: Serialize> {
    pub table: String,
    pub next_id: String,
    pub records: HashMap<String, T>,
}

//////////////////////
// Public functions //
//////////////////////

pub fn update_table<T: Serialize>(table: &str, t: &T) -> Result<()> {
    let serialized = try!(serde_json::to_string(&create_base_data(table, t)));
    let db_table = Path::new("./db").join(table);
    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

pub fn create_table<T: Serialize>(table: &str, t: &T) -> Result<()> {
    try!(create_db_dir());

    let serialized = try!(serde_json::to_string(&create_base_data(table, t)));
    let db_table = Path::new("./db").join(table);

    if db_table.exists() {
        return Ok(());
    }

    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

pub fn read_table(table: &str) -> Result<String> {
    let db_table = Path::new("./db").join(table);
    let mut file = match File::open(db_table) {
        Ok(f) => f,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(Error::NoSuchTable(table.to_owned()))
        }
        Err(e) => return Err(Error::Io(e)),
    };

    let mut buffer = String::new();
    try!(file.read_to_string(&mut buffer));

    Ok(buffer)
}

pub fn drop_table(table: &str) -> io::Result<()> {
    let t = Path::new("./db").join(table);
    try!(fs::remove_file(t));
    Ok(())
}

pub fn append_records<T: Serialize + Deserialize>(table: &str, t: T) -> Result<()> {
    let mut data = get_table(table);
    let increased_next_id = data.next_id.parse::<i32>().unwrap();
    let new_id = increased_next_id + 1;

    data.records.insert(increased_next_id.to_string(), t);
    data.next_id = new_id.to_string();

    upgrade_table(table, &data)
}

// This returns an actual type. Only to be used as a data manipulator. Not a `Result<()>`.
pub fn get_table<T: Serialize + Deserialize>(table: &str) -> Data<T> {
    let data: Data<T> = serde_json::from_str(&read_table(table).unwrap()).unwrap();
    data
}

// This returns the HashMap<String, T> of the given table. Not a `Result<()>`.
pub fn get_table_records<T: Serialize + Deserialize>(table: &str) -> HashMap<String, T> {
    get_table(table).records
}

///////////////////////
// Private functions //
///////////////////////

fn upgrade_table<T: Serialize>(table: &str, t: &T) -> Result<()> {
    let serialized = try!(serde_json::to_string(t));
    let db_table = Path::new("./db").join(table);
    let mut buffer = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

fn create_base_data<T: Serialize>(table: &str, t: T) -> Data<T> {
    let mut record = HashMap::new();
    record.insert("0".to_string(), t);

    let d = Data {
        table: table.to_string(),
        next_id: "1".to_string(),
        records: record,
    };

    d // Returns the Data<T> struct
}

fn create_db_dir() -> io::Result<()> {
    if Path::new("./db").exists() {
        return Ok(());
    }

    fs::create_dir("db")
}

///////////
// Tests //
///////////

#[cfg(test)]
mod tests {
    extern crate test;
    extern crate serde;

    use self::test::Bencher;

    use super::*;
    use sc;

    #[test]
    fn it_can_create_update_and_drop_a_table_and_take_any_struct_to_add_data() {
        let a = sc::Coordinates { x: 42, y: 9000 };
        let b = sc::Coordinates { x: 32, y: 8765 };
        let c = sc::Coordinates { x: 23, y: 900 };
        let d = sc::Coordinates { x: 105, y: 7382 };

        let e = "{\"table\":\"test\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":42,\"y\":9000}}}";
        let f = "{\"table\":\"test\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":32,\"y\":8765}}}";

        create_table("test", &a).unwrap();
        assert_eq!(e, read_table("test").unwrap());

        update_table("test", &b).unwrap();
        assert_eq!(f, read_table("test").unwrap());

        drop_table("test").unwrap();
        create_table("test", &a).unwrap();

        append_records("test", b).unwrap();
        append_records("test", c).unwrap();
        append_records("test", d).unwrap();

        assert!(read_table("test").unwrap().contains("2"));
        assert!(read_table("test").unwrap().contains("3"));
        assert!(read_table("test").unwrap().contains("4"));

        drop_table("test").unwrap();
    }

    #[test]
    // This is not a benchmark - it is just to make sure this can be done correctly
    fn it_can_create_100_tables_and_drop_them_all() {
        for n in 1..101 {
            let table = format!("{}", n);
            let a = sc::Coordinates { x: 42, y: 9000 };

            create_table(&*table, &a).unwrap();
        }

        for k in 1..101 {
            let table = format!("{}", k);

            drop_table(&*table).unwrap();
        }
    }

    #[bench]
    fn bench_update_table(b: &mut Bencher) {
        let object = sc::Coordinates { x: 42, y: 9000 };

        b.iter(|| update_table("test2", &object).unwrap());
    }
}
