// Copyright 2016 The Fe_Bucket Project Developers. See the COPYRIGHT file at
// the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This
// file may not be copied, modified, or distributed except according to those
// terms.

use serde::ser::{self, Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer};

#[derive(Debug, PartialEq)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
enum Field {
    X,
    Y
}

const FIELDS: &'static [&'static str] = &["x", "y"];

/// Visit the fields of `Coordinates` to serialize them
#[derive(Debug)]
struct SeVisitor<'a> {
    data: &'a Coordinates,
    field: usize
}

impl<'a> SeVisitor<'a> {
    fn new(data: &'a Coordinates) -> SeVisitor<'a> {
        SeVisitor { data: data, field: 0 }
    }
}

impl<'a> ser::MapVisitor for SeVisitor<'a> {
    fn visit<S: Serializer>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error> {
        match self.field {
            0 => {
                try!(serializer.serialize_struct_elt("x", self.data.x));
                self.field += 1;
                Ok(Some(()))
            },
            1 => {
                try!(serializer.serialize_struct_elt("y", self.data.y));
                self.field += 1;
                Ok(None)
            },
            _ => unreachable!()
        }
    }
}

impl Serialize for Coordinates {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_struct("Coordinates", SeVisitor::new(self))
    }
}

struct FieldVisitor;

impl de::Visitor for FieldVisitor {
    type Value = Field;

    fn visit_str<E: de::Error>(&mut self, value: &str) -> Result<Field, E> {
        match value {
            "x" => Ok(Field::X),
            "y" => Ok(Field::Y),
            _ => Err(de::Error::custom("Expected x or y"))
        }
    }
}

impl Deserialize for Field {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Field, D::Error> {
        deserializer.deserialize(FieldVisitor)
    }
}

struct DeVisitor;

impl de::Visitor for DeVisitor {
    type Value = Coordinates;

    fn visit_map<V: de::MapVisitor>(&mut self, mut visitor: V) -> Result<Coordinates, V::Error> {
        let mut x = None;
        let mut y = None;

        loop {
            match try!(visitor.visit_key()) {
                Some(Field::X) => {
                    x = Some(try!(visitor.visit_value()));
                },
                Some(Field::Y) => {
                    y = Some(try!(visitor.visit_value()));
                },
                None => { break; }
            }
        }

        let x = match x {
            Some(v) => v,
            None => { return visitor.missing_field("x"); }
        };
        let y = match y {
            Some(v) => v,
            None => { return visitor.missing_field("y"); }
        };

        try!(visitor.end());

        Ok(Coordinates { x: x, y: y})
    }
}
            
impl Deserialize for Coordinates {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Coordinates, D::Error> {
        deserializer.deserialize_struct("Coordinates", FIELDS, DeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn serialization_works() {
        let src = Coordinates { x: 42, y: 43 };
        let expected = r#"{"x":42,"y":43}"#;

        assert_eq!(expected, serde_json::to_string(&src).unwrap());
    }

    #[test]
    fn deserialization_works() {
        let src = r#"{
  "x": 42,
  "y": 43
}"#;
        let expected = Coordinates { x: 42, y: 43 };

        assert_eq!(expected, serde_json::from_str(src).unwrap());
    }
}
