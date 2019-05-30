extern crate base64;
extern crate libflate;
extern crate xml;
#[cfg(feature = "serde")]
use serde::de::{self, Deserialize, Deserializer};

mod color;
mod error;
mod image;
mod layer;
#[macro_use]
pub mod macros;
mod map;
pub mod object;
mod property;
mod tile;
mod tileset;

pub use self::{
    color::Color,
    error::Error,
    image::Image,
    layer::{ImageLayer, Layer},
    map::{parse_data, Map},
    property::{Properties, PropertyValue},
    tile::{Frame, Tile},
    tileset::Tileset,
};
