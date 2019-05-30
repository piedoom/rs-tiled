use crate::{
    error::Error, get_attrs, image::Image, object::ObjectGroup, parse_tag,
    property::parse_properties, property::Properties,
};
use std::{collections::HashMap, io::Read};
use xml::{attribute::OwnedAttribute, EventReader};

#[derive(Debug, PartialEq, Clone)]
pub struct Tile {
    pub id: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub images: Vec<Image>,
    pub properties: Properties,
    pub objectgroup: Option<ObjectGroup>,
    pub animation: Option<Vec<Frame>>,
    pub tile_type: Option<String>,
    pub probability: f32,
}

const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x8;
const FLIPPED_VERTICALLY_FLAG: u32 = 0x4;
const FLIPPED_DIAGONALLY_FLAG: u32 = 0x2;
const ALL_FLIP_FLAGS: u32 =
    FLIPPED_HORIZONTALLY_FLAG | FLIPPED_VERTICALLY_FLAG | FLIPPED_DIAGONALLY_FLAG;

impl Tile {
    pub fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<Tile, Error> {
        let ((tile_type, probability), id) = get_attrs!(
            attrs,
            optionals: [
                ("type", tile_type, |v:String| v.parse().ok()),
                ("probability", probability, |v:String| v.parse().ok()),
            ],
            required: [
                ("id", id, |v:String| v.parse::<u32>().ok()),
            ],
            Error::MalformedAttributes("tile must have an id with the correct type".to_string())
        );

        let flags = (id & ALL_FLIP_FLAGS) >> 28;
        let id: u32 = id & !ALL_FLIP_FLAGS;
        let diagon = flags & FLIPPED_DIAGONALLY_FLAG == FLIPPED_DIAGONALLY_FLAG;
        let flip_h = (flags & FLIPPED_HORIZONTALLY_FLAG == FLIPPED_HORIZONTALLY_FLAG) ^ diagon;
        let flip_v = (flags & FLIPPED_VERTICALLY_FLAG == FLIPPED_VERTICALLY_FLAG) ^ diagon;

        let mut images = Vec::new();
        let mut properties = HashMap::new();
        let mut objectgroup = None;
        let mut animation = None;
        parse_tag!(parser, "tile", {
            "image" => |attrs| {
                images.push(Image::new(parser, attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "objectgroup" => |attrs| {
                objectgroup = Some(ObjectGroup::new(parser, attrs, None)?);
                Ok(())
            },
            "animation" => |_| {
                animation = Some(parse_animation(parser)?);
                Ok(())
            },
        });
        Ok(Tile {
            id,
            flip_h,
            flip_v,
            images,
            properties,
            objectgroup,
            animation,
            tile_type,
            probability: probability.unwrap_or(1.0),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
    tile_id: u32,
    duration: u32,
}

impl Frame {
    pub fn new(attrs: Vec<OwnedAttribute>) -> Result<Frame, Error> {
        let ((), (tile_id, duration)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("tileid", tile_id, |v:String| v.parse().ok()),
                ("duration", duration, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("A frame must have tileid and duration".to_string())
        );
        Ok(Frame {
            tile_id: tile_id,
            duration: duration,
        })
    }
}

fn parse_animation<R: Read>(parser: &mut EventReader<R>) -> Result<Vec<Frame>, Error> {
    let mut animation = Vec::new();
    parse_tag!(parser, "animation", {
        "frame" => |attrs| {
            animation.push(Frame::new(attrs)?);
            Ok(())
        },
    });
    Ok(animation)
}
