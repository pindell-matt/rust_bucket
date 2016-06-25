// Copyright 2016 The Fe_Bucket Project Developers. See the COPYRIGHT file at
// the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This
// file may not be copied, modified, or distributed except according to those
// terms.

#![cfg_attr(all(test, feature = "benchmarks"), feature(test))]

extern crate serde_json;
extern crate serde;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::marker::PhantomData;
use serde::de::{self, Deserialize, Deserializer};
use std::io::prelude::*;
use std::result::Result as StdResult;
use serde::ser::{self, Serialize, Serializer};
use std::collections::HashMap;

// sc is the user defined schema
mod sc;

pub mod errors;
use errors::{Result, Error};

/// The table structure.
#[derive(PartialEq, Debug)]
pub struct Data<T> {
    pub table: String,
    pub next_id: String,
    pub records: HashMap<String, T>,
}

/// Fields of the `Data` type; support for deserialization.
#[derive(Debug)]
enum DataField {
    Table,
    NextId,
    Records
}

/// Fields of the `Data` type, as strings; support for deserialization
const DATA_FIELDS: &'static [&'static str] = &["table", "next_id", "records"];

/// Visit the various fields of `Data`, to serialize them.
#[derive(Debug)]
struct DataSeVisitor<'a, T: 'a> {
    data: &'a Data<T>,
    field: usize
}

impl<'a, T> DataSeVisitor<'a, T> {
    fn new(data: &'a Data<T>) -> DataSeVisitor<'a, T> {
        DataSeVisitor { data: data, field: 0 }
    }
}

impl<'a, T: Serialize> ser::MapVisitor for DataSeVisitor<'a, T>
{
    fn visit<S: Serializer>(&mut self, serializer: &mut S) -> StdResult<Option<()>, S::Error> {
        match self.field {
            0 => {
                try!(serializer.serialize_struct_elt("table", &self.data.table[..]));
                self.field += 1;
                Ok(Some(()))
            },
            1 => {
                try!(serializer.serialize_struct_elt("next_id", &self.data.next_id[..]));
                self.field += 1;
                Ok(Some(()))
            },
            2 => {
                try!(serializer.serialize_struct_elt("records", &self.data.records));
                self.field += 1;
                Ok(None)
            },
            _ => unreachable!()
        }
    }
}

impl<T: Serialize> Serialize for Data<T> {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> StdResult<(), S::Error> {
        serializer.serialize_struct("Data", DataSeVisitor::new(self))
    }
}

/// Marker struct for visiting data fields
struct DataFieldVisitor;

impl de::Visitor for DataFieldVisitor {
    type Value = DataField;

    fn visit_str<E: de::Error>(&mut self, value: &str) -> StdResult<DataField, E> {
        match value {
            "table" => Ok(DataField::Table),
            "next_id" => Ok(DataField::NextId),
            "records" => Ok(DataField::Records),
            _ => Err(de::Error::custom("Expected table, next_id, or records"))
        }
    }
}

impl Deserialize for DataField {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> StdResult<DataField, D::Error> {
        deserializer.deserialize(DataFieldVisitor)
    }
}

/// Marker struct for deserialization
struct DataDeVisitor<T>{
    _spook: PhantomData<T>
}

impl<T: Deserialize> de::Visitor for DataDeVisitor<T> {
    type Value = Data<T>;

    fn visit_map<V: de::MapVisitor>(&mut self, mut visitor: V) -> StdResult<Data<T>, V::Error> {
        let mut table = None;
        let mut next = None;
        let mut records = None;

        loop {
            match try!(visitor.visit_key()) {
                Some(DataField::Table) => {
                    table = Some(try!(visitor.visit_value()));
                },
                Some(DataField::NextId) => {
                    next = Some(try!(visitor.visit_value()));
                },
                Some(DataField::Records) => {
                    records = Some(try!(visitor.visit_value()));
                },
                None => { break; }
            }
        }

        let table = match table {
            Some(t) => t,
            None => { return visitor.missing_field("table"); }
        };
        let next = match next {
            Some(n) => n,
            None => { return visitor.missing_field("next_id"); }
        };
        let records = match records {
            Some(r) => r,
            None => { return visitor.missing_field("records"); }
        };

        try!(visitor.end());

        Ok(Data { table: table, next_id: next, records: records })
    }
}

impl<T: Deserialize> Deserialize for Data<T> {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> StdResult<Data<T>, D::Error> {
        deserializer.deserialize_struct("Data", DATA_FIELDS, DataDeVisitor { _spook: PhantomData })
    }
}

// Public functions *******************************************************************************
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

pub fn get_table<T: Serialize + Deserialize>(table: &str) -> Data<T> {
    let data: Data<T> = serde_json::from_str(&read_table(table).unwrap()).unwrap();
    data
}

pub fn get_table_records<T: Serialize + Deserialize>(table: &str) -> HashMap<String, T> {
    get_table(table).records
}

pub fn find<T: Serialize + Deserialize>(table: &str, id: &str) -> T {
    get_table_records(table).remove(id).unwrap()
}

pub fn delete<T: Serialize + Deserialize>(table: &str, id: &str) -> Result<()> {
    let mut current_table: HashMap<String, T> = get_table_records(table);
    current_table.remove(id);
    update_table(table, &current_table).unwrap();

    Ok(())
}

pub fn json_find<T: Serialize + Deserialize>(table: &str, id: &str) -> String {
    let incoming_record: T = find(table, id);
    let json_record = serde_json::to_string(&incoming_record);
    json_record.unwrap()
}

pub fn json_table_records<T: Serialize + Deserialize>(table: &str) -> String {
    let records: HashMap<String, T> = get_table_records(table);
    let json_records = serde_json::to_string(&records);
    json_records.unwrap()
}

pub fn json_table<T: Serialize + Deserialize>(table: &str) -> String {
    read_table(table).unwrap()
}

// Private functions ******************************************************************************

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

    let table_data = Data {
        table: table.to_string(),
        next_id: "1".to_string(),
        records: record,
    };

    table_data
}

fn create_db_dir() -> io::Result<()> {
    if Path::new("./db").exists() {
        return Ok(());
    }

    fs::create_dir("db")
}

// Tests ******************************************************************************************

#[cfg(test)]
mod tests {
    #[cfg(feature = "benchmarks")]
    extern crate test;

    #[cfg(feature = "benchmarks")]
    use self::test::Bencher;

    use super::*;
    use sc;

    use std::collections::HashMap;
    use serde_json;

    #[test]
    fn serialization_works() {
        let mut rec = HashMap::new();
        rec.insert("0".to_owned(), sc::Coordinates{x: 42, y: 43});

        let data = Data {
            table: "test".to_owned(),
            next_id: "1".to_owned(),
            records: rec
        };

        let expected = r#"{"table":"test","next_id":"1","records":{"0":{"x":42,"y":43}}}"#;

        assert_eq!(expected, serde_json::to_string(&data).unwrap());
    }

    #[test]
    fn deserialization_works() {
        let source = r#"{
  "table": "test",
  "next_id": "1",
  "records": {
    "0": {
      "x": 42,
      "y": 43
    }
  }
}"#;

        let mut rec = HashMap::new();
        rec.insert("0".to_owned(), sc::Coordinates{x: 42, y: 43});

        let expected = Data {
            table: "test".to_owned(),
            next_id: "1".to_owned(),
            records: rec
        };

        assert_eq!(expected, serde_json::from_str(source).unwrap());
    }

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

    #[test]
    fn it_can_get_and_find() {
        let a = sc::Coordinates { x: 42, y: 9000 };

        create_table("test3", &a).unwrap();

        assert_eq!(a, find("test3", "0"));

        drop_table("test3").unwrap();
    }

    #[test]
    fn it_can_return_json() {
        let a = sc::Coordinates { x: 42, y: 9000 };
        create_table("test5", &a).unwrap();
        assert_eq!(a, find("test5", "0"));

        let b: String = json_table::<sc::Coordinates>("test5");
        let c: String = json_table_records::<sc::Coordinates>("test5");
        let d: String = json_find::<sc::Coordinates>("test5", "0");

        let j = "{\"table\":\"test5\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":42,\"y\":9000}}}";
        assert_eq!(j, b);

        let k = "{\"0\":{\"x\":42,\"y\":9000}}";
        assert_eq!(k, c);

        let l = "{\"x\":42,\"y\":9000}";
        assert_eq!(l, d);

        drop_table("test5").unwrap();
    }

    #[test]
    fn it_can_delete_table_data_by_id() {
        let a = sc::Coordinates { x: 42, y: 9000 };

        create_table("test6", &a).unwrap();

        assert_eq!(a, find("test6", "0"));

        let del = delete::<sc::Coordinates>;
        del("test6", "0").unwrap();

        let jtable = json_table::<sc::Coordinates>;
        let table = jtable("test6");
        assert_eq!(table, "{\"table\":\"test6\",\"next_id\":\"1\",\"records\":{\"0\":{}}}");

        drop_table("test6").unwrap();
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_create_table(b: &mut Bencher) {
        let object = sc::Coordinates { x: 42, y: 9000 };

        b.iter(|| create_table("test4", &object).unwrap());
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_update_table(b: &mut Bencher) {
        let object = sc::Coordinates { x: 42, y: 9000 };

        b.iter(|| update_table("test2", &object).unwrap());
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_read_table(b: &mut Bencher) {
        b.iter(|| read_table("test2").unwrap());
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_json_table(b: &mut Bencher) {
        let a = json_table::<sc::Coordinates>;

        b.iter(|| a("test2"));
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_json_table_records(b: &mut Bencher) {
        let a = json_table_records::<sc::Coordinates>;

        b.iter(|| a("test2"));
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_json_find(b: &mut Bencher) {
        let a = json_find::<sc::Coordinates>;

        b.iter(|| a("test2", "0"));
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_find(b: &mut Bencher) {
        let a = find::<sc::Coordinates>;
        b.iter(|| a("test2", "0"));
    }
}
