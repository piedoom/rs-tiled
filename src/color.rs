use crate::error::ParseTileError;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromStr for Color {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Color, ParseTileError> {
        let s = if s.starts_with("#") { &s[1..] } else { s };
        if s.len() != 6 {
            return Err(ParseTileError::ColorError);
        }
        let r = u8::from_str_radix(&s[0..2], 16);
        let g = u8::from_str_radix(&s[2..4], 16);
        let b = u8::from_str_radix(&s[4..6], 16);
        if r.is_ok() && g.is_ok() && b.is_ok() {
            return Ok(Color {
                red: r.unwrap(),
                green: g.unwrap(),
                blue: b.unwrap(),
            });
        }
        Err(ParseTileError::ColorError)
    }
}
