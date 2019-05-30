use crate::{
    color::Color,
    error::{Error, ParseTileError},
    layer::{ImageLayer, Layer},
    object::ObjectGroup,
    property::{parse_properties, Properties},
    tileset::Tileset,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    str::FromStr,
};
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

#[cfg(feature = "amethyst")]
use specs::storage::{VecStorage, UnprotectedStorage};
#[cfg(feature = "amethyst")]
use amethyst_assets::{Asset, ProcessingState, Handle};


/// All Tiled files will be parsed into this. Holds all the layers and tilesets
#[derive(Debug, PartialEq, Clone)]
pub struct Map {
    pub version: String,
    pub orientation: Orientation,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tilesets: Vec<Tileset>,
    pub layers: Vec<Layer>,
    pub image_layers: Vec<ImageLayer>,
    pub object_groups: Vec<ObjectGroup>,
    pub properties: Properties,
    pub background_color: Option<Color>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            version: String::new(),
            orientation: Orientation::Orthogonal,
            width: 0,
            height: 0,
            tile_width: 0,
            tile_height: 0,
            tilesets: vec![],
            layers: vec![],
            image_layers: vec![],
            object_groups: vec![],
            properties: Properties::with_capacity(0),
            background_color: None,
        }
    }
}

unsafe impl Send for Map {}
unsafe impl Sync for Map {}

impl Map {
    fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Map, Error> {
        let (c, (v, o, w, h, tw, th)) = get_attrs!(
            attrs,
            optionals: [
                ("backgroundcolor", color, |v:String| v.parse().ok()),
            ],
            required: [
                ("version", version, |v| Some(v)),
                ("orientation", orientation, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("tilewidth", tile_width, |v:String| v.parse().ok()),
                ("tileheight", tile_height, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("map must have a version, width and height with correct types".to_string())
        );

        let mut tilesets = Vec::new();
        let mut layers = Vec::new();
        let mut image_layers = Vec::new();
        let mut properties = HashMap::new();
        let mut object_groups = Vec::new();
        let mut layer_index = 0;
        parse_tag!(parser, "map", {
            "tileset" => | attrs| {
                tilesets.push(Tileset::new(parser, attrs, map_path)?);
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(Layer::new(parser, attrs, w, layer_index)?);
                layer_index += 1;
                Ok(())
            },
            "imagelayer" => |attrs| {
                image_layers.push(ImageLayer::new(parser, attrs, layer_index)?);
                layer_index += 1;
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "objectgroup" => |attrs| {
                object_groups.push(ObjectGroup::new(parser, attrs, Some(layer_index))?);
                layer_index += 1;
                Ok(())
            },
        });
        Ok(Map {
            version: v,
            orientation: o,
            width: w,
            height: h,
            tile_width: tw,
            tile_height: th,
            tilesets,
            layers,
            image_layers,
            object_groups,
            properties,
            background_color: c,
        })
    }

    /// This function will return the correct Tileset given a GID.
    pub fn get_tileset_by_gid(&self, gid: u32) -> Option<&Tileset> {
        let mut maximum_gid: i32 = -1;
        let mut maximum_ts = None;
        for tileset in self.tilesets.iter() {
            if tileset.first_gid as i32 > maximum_gid && tileset.first_gid <= gid {
                maximum_gid = tileset.first_gid as i32;
                maximum_ts = Some(tileset);
            }
        }
        maximum_ts
    }

    /// Parse a buffer hopefully containing the contents of a Tiled file and try to
    /// parse it.
    pub fn parse<R: Read>(reader: R) -> Result<Map, Error> {
        Self::parse_impl(reader, None)
    }

    /// Parse a file hopefully containing a Tiled map and try to parse it.  If the
    /// file has an external tileset, the tileset file will be loaded using a path
    /// relative to the map file's path.
    pub fn parse_file(path: &Path) -> Result<Map, Error> {
        let file = File::open(path)
            .map_err(|_| Error::Other(format!("Map file not found: {:?}", path)))?;
        Self::parse_impl(file, Some(path))
    }

    /// Parse a buffer hopefully containing the contents of a Tiled file and try to
    /// parse it. This augments `parse` with a file location: some engines
    /// (e.g. Amethyst) simply hand over a byte stream (and file location) for parsing,
    /// in which case this function may be required.
    pub fn parse_with_path<R: Read>(reader: R, path: &Path) -> Result<Map, Error> {
        Self::parse_impl(reader, Some(path))
    }

    fn parse_impl<R: Read>(reader: R, map_path: Option<&Path>) -> Result<Map, Error> {
        let mut parser = EventReader::new(reader);
        loop {
            match parser.next().map_err(Error::XmlDecodingError)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "map" {
                        return Map::new(&mut parser, attributes, map_path);
                    }
                }
                XmlEvent::EndDocument => {
                    return Err(Error::PrematureEnd(
                        "Document ended before map was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }
}

#[cfg(feature = "amethyst")]
impl Asset for Map {
    const NAME: &'static str = "tiled::Map";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

#[cfg(feature = "amethyst")]
impl From<Map> for Result<ProcessingState<Map>, amethyst_error::Error> {
    fn from(map: Map)
        -> Result<ProcessingState<Map>, amethyst_error::Error> {
            Ok(ProcessingState::Loaded(Map::default()))
    }
}


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

impl FromStr for Orientation {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Orientation, ParseTileError> {
        match s {
            "orthogonal" => Ok(Orientation::Orthogonal),
            "isometric" => Ok(Orientation::Isometric),
            "staggered" => Ok(Orientation::Staggered),
            "hexagonal" => Ok(Orientation::Hexagonal),
            _ => Err(ParseTileError::OrientationError),
        }
    }
}

pub fn parse_data<R: Read>(
    parser: &mut EventReader<R>,
    attrs: Vec<OwnedAttribute>,
    width: u32,
) -> Result<Vec<Vec<u32>>, Error> {
    let ((e, c), ()) = get_attrs!(
        attrs,
        optionals: [
            ("encoding", encoding, |v| Some(v)),
            ("compression", compression, |v| Some(v)),
        ],
        required: [],
        Error::MalformedAttributes("data must have an encoding and a compression".to_string())
    );

    match (e, c) {
        (None, None) => {
            return Err(Error::Other(
                "XML format is currently not supported".to_string(),
            ))
        }
        (Some(e), None) => match e.as_ref() {
            "base64" => return parse_base64(parser).map(|v| convert_to_u32(&v, width)),
            "csv" => return decode_csv(parser),
            e => return Err(Error::Other(format!("Unknown encoding format {}", e))),
        },
        (Some(e), Some(c)) => match (e.as_ref(), c.as_ref()) {
            ("base64", "zlib") => {
                return parse_base64(parser)
                    .and_then(decode_zlib)
                    .map(|v| convert_to_u32(&v, width))
            }
            ("base64", "gzip") => {
                return parse_base64(parser)
                    .and_then(decode_gzip)
                    .map(|v| convert_to_u32(&v, width))
            }
            (e, c) => {
                return Err(Error::Other(format!(
                    "Unknown combination of {} encoding and {} compression",
                    e, c
                )))
            }
        },
        _ => return Err(Error::Other("Missing encoding format".to_string())),
    };
}

fn parse_base64<R: Read>(parser: &mut EventReader<R>) -> Result<Vec<u8>, Error> {
    loop {
        match parser.next().map_err(Error::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                return base64::decode(s.trim().as_bytes()).map_err(Error::Base64DecodingError)
            }
            XmlEvent::EndElement { name, .. } => {
                if name.local_name == "data" {
                    return Ok(Vec::new());
                }
            }
            _ => {}
        }
    }
}

fn decode_zlib(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    use libflate::zlib::Decoder;
    let mut zd =
        Decoder::new(BufReader::new(&data[..])).map_err(|e| Error::DecompressingError(e))?;
    let mut data = Vec::new();
    match zd.read_to_end(&mut data) {
        Ok(_v) => {}
        Err(e) => return Err(Error::DecompressingError(e)),
    }
    Ok(data)
}

fn decode_gzip(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    use libflate::gzip::Decoder;
    let mut zd =
        Decoder::new(BufReader::new(&data[..])).map_err(|e| Error::DecompressingError(e))?;

    let mut data = Vec::new();
    zd.read_to_end(&mut data)
        .map_err(|e| Error::DecompressingError(e))?;
    Ok(data)
}

fn decode_csv<R: Read>(parser: &mut EventReader<R>) -> Result<Vec<Vec<u32>>, Error> {
    loop {
        match parser.next().map_err(Error::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                let mut rows: Vec<Vec<u32>> = Vec::new();
                for row in s.split('\n') {
                    if row.trim() == "" {
                        continue;
                    }
                    rows.push(
                        row.split(',')
                            .filter(|v| v.trim() != "")
                            .map(|v| v.replace('\r', "").parse().unwrap())
                            .collect(),
                    );
                }
                return Ok(rows);
            }
            XmlEvent::EndElement { name, .. } => {
                if name.local_name == "data" {
                    return Ok(Vec::new());
                }
            }
            _ => {}
        }
    }
}

fn convert_to_u32(all: &Vec<u8>, width: u32) -> Vec<Vec<u32>> {
    let mut data = Vec::new();
    for chunk in all.chunks((width * 4) as usize) {
        let mut row = Vec::new();
        for i in 0..width {
            let start: usize = i as usize * 4;
            let n = ((chunk[start + 3] as u32) << 24)
                + ((chunk[start + 2] as u32) << 16)
                + ((chunk[start + 1] as u32) << 8)
                + chunk[start] as u32;
            row.push(n);
        }
        data.push(row);
    }
    data
}
