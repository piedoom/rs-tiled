use crate::{error::Error, get_attrs, image::Image, parse_tag, tile::Tile};
use std::{fs::File, io::Read, path::Path};
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

/// A tileset, usually the tilesheet image.
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
    /// The GID of the first tile stored
    pub first_gid: u32,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    /// The Tiled spec says that a tileset can have mutliple images so a `Vec`
    /// is used. Usually you will only use one.
    pub images: Vec<Image>,
    pub tiles: Vec<Tile>,
}

impl Tileset {
    pub fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Tileset, Error> {
        Tileset::new_internal(parser, &attrs).or_else(|_| Tileset::new_reference(&attrs, map_path))
    }

    fn new_internal<R: Read>(
        parser: &mut EventReader<R>,
        attrs: &Vec<OwnedAttribute>,
    ) -> Result<Tileset, Error> {
        let ((spacing, margin), (first_gid, name, width, height)) = get_attrs!(
           attrs,
           optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
            ],
           required: [
                ("firstgid", first_gid, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let mut images = Vec::new();
        let mut tiles = Vec::new();
        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                images.push(Image::new(parser, attrs)?);
                Ok(())
            },
            "tile" => |attrs| {
                tiles.push(Tile::new(parser, attrs)?);
                Ok(())
            },
        });

        Ok(Tileset {
            first_gid: first_gid,
            name: name,
            tile_width: width,
            tile_height: height,
            spacing: spacing.unwrap_or(0),
            margin: margin.unwrap_or(0),
            images: images,
            tiles: tiles,
        })
    }

    fn new_reference(
        attrs: &Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Tileset, Error> {
        let ((), (first_gid, source)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("firstgid", first_gid, |v:String| v.parse().ok()),
                ("source", name, |v| Some(v)),
            ],
            Error::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let tileset_path = map_path.ok_or(Error::Other("Maps with external tilesets must know their file location.  See parse_with_path(Path).".to_string()))?.with_file_name(source);
        let file = File::open(&tileset_path).map_err(|_| {
            Error::Other(format!(
                "External tileset file not found: {:?}",
                tileset_path
            ))
        })?;
        Tileset::new_external(file, first_gid)
    }

    fn new_external<R: Read>(file: R, first_gid: u32) -> Result<Tileset, Error> {
        let mut tileset_parser = EventReader::new(file);
        loop {
            match tileset_parser.next().map_err(Error::XmlDecodingError)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "tileset" {
                        return Tileset::parse_external_tileset(
                            first_gid,
                            &mut tileset_parser,
                            &attributes,
                        );
                    }
                }
                XmlEvent::EndDocument => {
                    return Err(Error::PrematureEnd(
                        "Tileset Document ended before map was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    fn parse_external_tileset<R: Read>(
        first_gid: u32,
        parser: &mut EventReader<R>,
        attrs: &Vec<OwnedAttribute>,
    ) -> Result<Tileset, Error> {
        let ((spacing, margin), (name, width, height)) = get_attrs!(
            attrs,
            optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
            ],
            required: [
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let mut images = Vec::new();
        let mut tiles = Vec::new();
        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                images.push(Image::new(parser, attrs)?);
                Ok(())
            },
            "tile" => |attrs| {
                tiles.push(Tile::new(parser, attrs)?);
                Ok(())
            },
        });

        Ok(Tileset {
            first_gid: first_gid,
            name: name,
            tile_width: width,
            tile_height: height,
            spacing: spacing.unwrap_or(0),
            margin: margin.unwrap_or(0),
            images: images,
            tiles: tiles,
        })
    }

    /// Parse a buffer hopefully containing the contents of a Tiled tileset.
    ///
    /// External tilesets do not have a firstgid attribute.  That lives in the
    /// map. You must pass in `first_gid`.  If you do not need to use gids for anything,
    /// passing in 1 will work fine.
    pub fn parse_tileset<R: Read>(reader: R, first_gid: u32) -> Result<Tileset, Error> {
        Tileset::new_external(reader, first_gid)
    }
}
