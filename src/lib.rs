#![feature(custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate serde;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use serde::Deserialize;
use std::io::BufReader;
use std::io::prelude::*;
use serde::ser::Serialize;
use std::collections::HashMap;

mod sc; // sc is the user defined schema

pub mod errors;

// private struct for the db
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Data<T: Serialize>{
    pub table:   String,
    pub next_id: String,
    pub records: HashMap<String, T>,
}

// public functions first then private functions

pub fn update_table<T: Serialize>(table: &str, t: &T) -> io::Result<()> {
    let     serialized = serde_json::to_string(&create_base_data(table.clone(), t)).unwrap();
    let     db_table   = Path::new("./db").join(table);
    let mut buffer     = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

pub fn create_table<T: Serialize>(table: &str, t: &T) -> io::Result<()> {
    try!(create_db_dir());

    let serialized = serde_json::to_string(&create_base_data(table, t)).unwrap();
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

#[allow(dead_code)]
pub fn drop_table(table: &str) -> io::Result<()> {
    let t = Path::new("./db").join(table);
    try!(fs::remove_file(t));
    Ok(())
}

#[allow(unused_must_use)]
pub fn append_records<T: Serialize + Deserialize>(table: &str, t: T) -> io::Result<()> {
    let mut data: Data<_>     = serde_json::from_str(&read_table("test")).unwrap();
    let     increased_next_id = data.next_id.parse::<i32>().unwrap();
    let     new_id            = increased_next_id + 1;

    data.records.insert(increased_next_id.to_string(), t);
    data.next_id = new_id.to_string();

    upgrade_table(table, &data)
}

// private functions and tests

fn upgrade_table<T: Serialize>(table: &str, t: &T) -> io::Result<()> {
    let     serialized = serde_json::to_string(t).unwrap();
    let     db_table   = Path::new("./db").join(table);
    let mut buffer     = try!(File::create(db_table));
    try!(buffer.write_all(serialized.as_bytes()));

    Ok(())
}

#[allow(dead_code, unused_must_use)]
fn read_records<T: Serialize + Deserialize>() -> HashMap<String, T> {
    let data: Data<_> = serde_json::from_str(&read_table("test")).unwrap();
    data.records
}

fn create_base_data<T: Serialize>(table: &str, t: T) -> Data<T> {
    let mut record = HashMap::new();
    record.insert("0".to_string(), t);

    let d = Data {
        table:   table.to_string(),
        next_id: "1".to_string(),
        records: record,
    };

    d // return the Data<T> struct
}

fn create_db_dir() -> io::Result<()>{
    if Path::new("./db").exists() {
        return Ok(())
    }

    fs::create_dir("db")
}

#[cfg(test)]
mod tests {
    extern crate test;
    extern crate serde;

    use self::test::Bencher;

    use super::*;
    use sc;

    #[test]
    #[allow(unused_must_use)]
    fn it_can_create_update_and_drop_a_table_and_take_any_struct_to_add_data() {
        let a = sc::Coordinates {x: 42, y: 9000};
        let b = sc::Coordinates {x: 32, y: 8765};
        let c = sc::Coordinates {x: 23, y: 900};
        let d = sc::Coordinates {x: 105, y: 7382};

        let ex_1 = "{\"table\":\"test\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":42,\"y\":9000}}}";
        let ex_2 = "{\"table\":\"test\",\"next_id\":\"1\",\"records\":{\"0\":{\"x\":32,\"y\":8765}}}";

        create_table("test", &a).unwrap();
        assert_eq!(ex_1, read_table("test".to_string()));

        update_table("test", &b).unwrap();
        assert_eq!(ex_2, read_table("test".to_string()));

        drop_table("test");
        create_table("test", &a).unwrap();

        append_records("test", b);
        append_records("test", c);
        append_records("test", d);

        assert!(read_table("test".to_string()).contains("2"));
        assert!(read_table("test".to_string()).contains("3"));
        assert!(read_table("test".to_string()).contains("4"));

        drop_table("test");
    }

    #[test]
    #[allow(unused_must_use)]
    // This is not a benchmark - it is just to make sure this can be done correctly
    fn it_can_create_100_tables_and_drop_them_all() {
        for n in 1..101 {
            let table = format!("{}", n);
            let a     = sc::Coordinates {x: 42, y: 9000};

            create_table(&*table, &a).unwrap();
        }

        for k in 1..101 {
            let table = format!("{}", k);

            drop_table(&*table);
        }
    }

    #[bench]
    fn bench_update_table(b: &mut Bencher) {
        let object = sc::Coordinates {x: 42, y: 9000};

        b.iter(|| update_table("test", &object).unwrap());
    }
}
