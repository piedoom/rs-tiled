use crate::{
    color::Color,
    error::Error,
    property::{parse_properties, Properties},
};
use std::{collections::HashMap, io::Read};
use xml::{attribute::OwnedAttribute, reader::EventReader};

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectGroup {
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
    pub objects: Vec<Object>,
    pub color: Option<Color>,
    /**
     * Layer index is not preset for tile collision boxes
     */
    pub layer_index: Option<u32>,
}

impl ObjectGroup {
    pub fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        layer_index: Option<u32>,
    ) -> Result<ObjectGroup, Error> {
        let ((o, v, c, n), ()) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("color", color, |v:String| v.parse().ok()),
                ("name", name, |v:String| v.into()),
            ],
            required: [],
            Error::MalformedAttributes("object groups must have a name".to_string())
        );
        let mut objects = Vec::new();
        parse_tag!(parser, "objectgroup", {
            "object" => |attrs| {
                objects.push(Object::new(parser, attrs)?);
                Ok(())
            },
        });
        Ok(ObjectGroup {
            name: n.unwrap_or(String::new()),
            opacity: o.unwrap_or(1.0),
            visible: v.unwrap_or(true),
            objects: objects,
            color: c,
            layer_index,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectShape {
    Rect { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polyline { points: Vec<(f32, f32)> },
    Polygon { points: Vec<(f32, f32)> },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub id: u32,
    pub gid: u32,
    pub name: String,
    pub obj_type: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub visible: bool,
    pub shape: ObjectShape,
    pub properties: Properties,
}

impl Object {
    pub fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<Object, Error> {
        let ((id, gid, n, t, w, h, v, r), (x, y)) = get_attrs!(
            attrs,
            optionals: [
                ("id", id, |v:String| v.parse().ok()),
                ("gid", gid, |v:String| v.parse().ok()),
                ("name", name, |v:String| v.parse().ok()),
                ("type", obj_type, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok()),
                ("rotation", rotation, |v:String| v.parse().ok()),
            ],
            required: [
                ("x", x, |v:String| v.parse().ok()),
                ("y", y, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("objects must have an x and a y number".to_string())
        );
        let v = v.unwrap_or(true);
        let w = w.unwrap_or(0f32);
        let h = h.unwrap_or(0f32);
        let r = r.unwrap_or(0f32);
        let id = id.unwrap_or(0u32);
        let gid = gid.unwrap_or(0u32);
        let n = n.unwrap_or(String::new());
        let t = t.unwrap_or(String::new());
        let mut shape = None;
        let mut properties = HashMap::new();

        parse_tag!(parser, "object", {
            "ellipse" => |_| {
                shape = Some(ObjectShape::Ellipse {
                    width: w,
                    height: h,
                });
                Ok(())
            },
            "polyline" => |attrs| {
                shape = Some(Object::new_polyline(attrs)?);
                Ok(())
            },
            "polygon" => |attrs| {
                shape = Some(Object::new_polygon(attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        let shape = shape.unwrap_or(ObjectShape::Rect {
            width: w,
            height: h,
        });

        Ok(Object {
            id: id,
            gid: gid,
            name: n.clone(),
            obj_type: t.clone(),
            x: x,
            y: y,
            rotation: r,
            visible: v,
            shape: shape,
            properties: properties,
        })
    }

    pub fn new_polyline(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape, Error> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("points", points, |v| Some(v)),
            ],
            Error::MalformedAttributes("A polyline must have points".to_string())
        );
        let points = Object::parse_points(s)?;
        Ok(ObjectShape::Polyline { points: points })
    }

    pub fn new_polygon(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape, Error> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("points", points, |v| Some(v)),
            ],
            Error::MalformedAttributes("A polygon must have points".to_string())
        );
        let points = Object::parse_points(s)?;
        Ok(ObjectShape::Polygon { points: points })
    }

    pub fn parse_points(s: String) -> Result<Vec<(f32, f32)>, Error> {
        let pairs = s.split(' ');
        let mut points = Vec::new();
        for v in pairs.map(|p| p.split(',')) {
            let v: Vec<&str> = v.collect();
            if v.len() != 2 {
                return Err(Error::MalformedAttributes(
                    "one of a polyline's points does not have an x and y coordinate".to_string(),
                ));
            }
            let (x, y) = (v[0].parse().ok(), v[1].parse().ok());
            if x.is_none() || y.is_none() {
                return Err(Error::MalformedAttributes(
                    "one of polyline's points does not have i32eger coordinates".to_string(),
                ));
            }
            points.push((x.unwrap(), y.unwrap()));
        }
        Ok(points)
    }
}
