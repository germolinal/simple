/*
MIT License

Copyright (c)  Germán Molina

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Reads `Point3D` and `Vector3D` written either as `[x, y, z]` (as in SIMPLE
//! models) or as `{x, y, z}` (as they are serialized).
//!
//! A derived `Deserialize` accepts both only if the format lets the input
//! decide; json5 (since 1.0) only accepts `{...}` for structs.

use crate::Float;
use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// Deserializes the `(x, y, z)` components of a `name` (e.g., "Point3D")
pub(crate) fn deserialize_xyz<'de, D: Deserializer<'de>>(
    deserializer: D,
    name: &'static str,
) -> Result<(Float, Float, Float), D::Error> {
    deserializer.deserialize_any(XYZVisitor(name))
}

struct XYZVisitor(&'static str);

impl<'de> Visitor<'de> for XYZVisitor {
    type Value = (Float, Float, Float);

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a {} as [x, y, z] or {{x, y, z}}", self.0)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let x = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let y = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let z = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(4, &self));
        }
        Ok((x, y, z))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let (mut x, mut y, mut z) = (None, None, None);
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "x" => x = Some(map.next_value()?),
                "y" => y = Some(map.next_value()?),
                "z" => z = Some(map.next_value()?),
                // (like a derived `Deserialize`)
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok((
            x.ok_or_else(|| de::Error::missing_field("x"))?,
            y.ok_or_else(|| de::Error::missing_field("y"))?,
            z.ok_or_else(|| de::Error::missing_field("z"))?,
        ))
    }
}

#[cfg(test)]
mod testing {
    use crate::{Point3D, Vector3D};

    #[test]
    fn test_array_and_object() -> Result<(), String> {
        let expected = Point3D::new(1., 2., 0.);
        for src in ["[1, 2, 0]", "[1.0, 2.0, 0.0]", "{x: 1, y: 2, z: 0}"] {
            let p: Point3D = json5::from_str(src).map_err(|e| format!("{src}: {e}"))?;
            assert_eq!(p, expected, "{src}");
            let v: Vector3D = json5::from_str(src).map_err(|e| format!("{src}: {e}"))?;
            assert_eq!(Point3D::from(v), expected, "{src}");
        }

        // Round trip through serde_json
        let s = serde_json::to_string(&expected).map_err(|e| e.to_string())?;
        let p: Point3D = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        assert_eq!(p, expected);
        let p: Point3D = serde_json::from_str("[1, 2, 0]").map_err(|e| e.to_string())?;
        assert_eq!(p, expected);

        // Wrong number of components
        for src in ["[1, 2]", "[1, 2, 3, 4]", "{x: 1, y: 2}"] {
            assert!(json5::from_str::<Point3D>(src).is_err(), "{src}");
        }
        Ok(())
    }
}
