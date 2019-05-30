extern crate base64;
extern crate libflate;
extern crate xml;
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
#[cfg(feature = "amethyst")]
mod amethyst;

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

#[cfg(feature = "amethyst")]
pub use amethyst::TmxFormat;