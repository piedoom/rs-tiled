use crate::{color::Color, error::Error, get_attrs, parse_tag};
use std::io::Read;
use xml::{attribute::OwnedAttribute, EventReader};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Image {
    /// The filepath of the image
    pub source: String,
    pub width: i32,
    pub height: i32,
    pub transparent_color: Option<Color>,
}

impl Image {
    pub fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<Image, Error> {
        let (c, (s, w, h)) = get_attrs!(
            attrs,
            optionals: [
                ("trans", trans, |v:String| v.parse().ok()),
            ],
            required: [
                ("source", source, |v| Some(v)),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("image must have a source, width and height with correct types".to_string())
        );

        parse_tag!(parser, "image", { "" => |_| Ok(()) });
        Ok(Image {
            source: s,
            width: w,
            height: h,
            transparent_color: c,
        })
    }
}
