//! Integration with Amethyst assets
use amethyst::{
    assets::Format,
    Error, error::ResultExt,
};

use crate::map::Map;
use std::io::BufReader;

#[derive(Clone, Debug, Default)]
pub struct TmxFormat;

impl Format<Map> for TmxFormat
{
    fn name(&self) -> &'static str {
        "Tmx"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Map, Error> {
        let reader =
            BufReader::new(&*bytes);
        Map::parse(reader).map_err(|e| amethyst::Error::new(e))
    }
}
