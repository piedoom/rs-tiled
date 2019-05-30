use crate::{error::Error, get_attrs, parse_tag};
use std::collections::HashMap;
use std::io::Read;
use xml::{attribute::OwnedAttribute, reader::EventReader};

#[derive(Debug, PartialEq, Clone)]
pub enum PropertyValue {
    BoolValue(bool),
    FloatValue(f32),
    IntValue(i32),
    ColorValue(u32),
    StringValue(String),
}

impl PropertyValue {
    pub fn new(property_type: String, value: String) -> Result<PropertyValue, Error> {
        use std::error::Error as StdError;

        // Check the property type against the value.
        match property_type.as_str() {
            "bool" => match value.parse() {
                Ok(val) => Ok(PropertyValue::BoolValue(val)),
                Err(err) => Err(Error::Other(err.description().into())),
            },
            "float" => match value.parse() {
                Ok(val) => Ok(PropertyValue::FloatValue(val)),
                Err(err) => Err(Error::Other(err.description().into())),
            },
            "int" => match value.parse() {
                Ok(val) => Ok(PropertyValue::IntValue(val)),
                Err(err) => Err(Error::Other(err.description().into())),
            },
            "color" if value.len() > 1 => match u32::from_str_radix(&value[1..], 16) {
                Ok(color) => Ok(PropertyValue::ColorValue(color)),
                Err(_) => Err(Error::Other(format!("Improperly formatted color property"))),
            },
            "string" => Ok(PropertyValue::StringValue(value)),
            _ => Err(Error::Other(format!(
                "Unknown property type \"{}\"",
                property_type
            ))),
        }
    }
}

pub type Properties = HashMap<String, PropertyValue>;

pub fn parse_properties<R: Read>(parser: &mut EventReader<R>) -> Result<Properties, Error> {
    let mut p = HashMap::new();
    parse_tag!(parser, "properties", {
        "property" => |attrs:Vec<OwnedAttribute>| {
            let (t, (k, v)) = get_attrs!(
                attrs,
                optionals: [
                    ("type", property_type, |v| Some(v)),
                ],
                required: [
                    ("name", key, |v| Some(v)),
                    ("value", value, |v| Some(v)),
                ],
                Error::MalformedAttributes("property must have a name and a value".to_string())
            );
            let t = t.unwrap_or("string".into());

            p.insert(k, PropertyValue::new(t, v)?);
            Ok(())
        },
    });
    Ok(p)
}
