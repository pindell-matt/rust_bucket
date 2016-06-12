use serde::ser::Serialize;
use serde::de::Deserialize;

#[warn(unused_imports)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}
