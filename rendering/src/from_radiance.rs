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

use crate::colour::Spectrum;
use crate::Float;

use crate::material::{Dielectric, Diffuse, Glass, Light, Metal, Mirror, Plastic};

use crate::material::Material;
use crate::primitive::Primitive;
use crate::scene::Scene;

use geometry::{DistantSource3D, Loop3D, Point3D, Polygon3D, Sphere3D, Triangulation3D, Vector3D};

use std::fs;
const MIN_SPECULARITY: Float = 1e-5;
#[derive(Default)]
struct RadianceReader {
    current_char_index: usize,
    is_done: bool,
    modifiers: Vec<String>,
    line: usize,
}

impl RadianceReader {
    /// Panics, showing an error message `msg` and showing the line in
    /// which the error happened.
    fn error_here(&self, msg: String) -> ! {
        panic!("Error at line {}: {}", self.line, msg)
    }

    fn get_modifier_index(&self, name: &str) -> usize {
        for (i, mod_name) in self.modifiers.iter().enumerate() {
            if name == mod_name {
                return i;
            }
        }
        self.error_here(format!(
            "Unknown modifier '{}' in the scene ... known modifiers are {:?}",
            name, self.modifiers
        ));
    }

    /// Consumes the leading whitespaces in the source **only if it is an ASCII whitespace**.
    /// Returns a boolean indicating whether the scanner consumed something or not
    fn consume_whitespace(&mut self, source: &[u8]) -> bool {
        if source.is_empty() {
            // nothing to consume... we are done
            self.is_done = true;
        }

        if self.is_done {
            return false;
        }

        if source[self.current_char_index].is_ascii_whitespace() {
            self.consume_char(source)
        } else {
            false
        }
    }

    /// Consumes a single char in the source, **only if it is not an ASCII whitespace**. Returns a boolean
    /// indicating whether the scanner consumed something or not
    fn consume_non_white(&mut self, source: &[u8]) -> bool {
        if source.is_empty() {
            // nothing to consume... we are done
            self.is_done = true;
        }
        if self.is_done {
            return false;
        }
        if source[self.current_char_index].is_ascii_whitespace() {
            false
        } else {
            self.consume_char(source)
        }
    }

    /// Consumes a single char. Returns a boolean indicating whether the
    /// scanner consumed anything or not.
    fn consume_char(&mut self, source: &[u8]) -> bool {
        if source.is_empty() {
            // nothing to consume... we are done
            self.is_done = true;
        }
        if self.is_done {
            // nothing to scan
            return false;
        }
        if source[self.current_char_index] == b'\n' {
            // account for newline
            self.line += 1;
        }
        self.current_char_index += 1;
        if self.current_char_index == source.len() {
            self.is_done = true;
        }
        true
    }

    /// Consumes whitespaces until reaching the next token
    fn reach_next_token(&mut self, source: &[u8]) {
        loop {
            if !self.consume_whitespace(source) {
                break;
            }
        }
    }

    /// Skips whitespaces and then consumes a single token.
    fn consume_token(&mut self, source: &[u8]) -> Result<String, String> {
        self.reach_next_token(source);

        let start = self.current_char_index;
        loop {
            if !self.consume_non_white(source) {
                break;
            }
        }

        if start == self.current_char_index {
            Ok("".to_string()) // empty token
        } else {
            let ret = std::str::from_utf8(&source[start..self.current_char_index])
                .map_err(|e| e.to_string())?
                .to_string();
            Ok(ret)
        }
    }

    /// Consume object
    fn consume_object(&mut self, source: &[u8], scene: &mut Scene) -> Result<(), String> {
        self.reach_next_token(source);
        if self.is_done {
            return Ok(());
        }

        let modifier = self.consume_token(source)?;
        if self.is_done {
            self.error_here("Incorrect source... no data after 'modifier'".to_string());
        }
        let object_type = self.consume_token(source)?;
        if self.is_done {
            self.error_here("Incorrect source... no data after 'object_type'".to_string());
        }
        let name = self.consume_token(source)?;
        if self.is_done {
            self.error_here("Incorrect source... no data after 'name'".to_string());
        }
        match object_type.as_bytes() {
            // modifiers
            b"plastic" => self.consume_plastic(source, scene, &modifier, &name),
            b"metal" => self.consume_metal(source, scene, &modifier, &name),
            b"light" => self.consume_light(source, scene, &modifier, &name),
            b"mirror" => self.consume_mirror(source, scene, &modifier, &name),
            b"dielectric" => self.consume_dielectric(source, scene, &modifier, &name),
            b"glass" => self.consume_glass(source, scene, &modifier, &name),

            // objects
            b"sphere" => self.consume_sphere(source, scene, &modifier, &name),
            b"source" => self.consume_source(source, scene, &modifier, &name),
            b"polygon" => self.consume_polygon(source, scene, &modifier, &name),
            _ => {
                self.error_here(format!("Unsupported/unknown object_type '{}'", object_type));
            }
        }
    }

    /// Consumes a Metal material
    fn consume_metal(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        _modifier: &str,
        name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "5".to_string());
        let red = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let green = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let blue = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let specularity = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let roughness = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;

        self.modifiers.push(name.to_string());
        let colour = Spectrum([red, green, blue]);

        let metal = if specularity < MIN_SPECULARITY {
            Material::Diffuse(Diffuse { colour })
        } else {
            Material::Metal(Metal {
                colour: Spectrum([red, green, blue]),
                specularity,
                roughness,
            })
        };
        scene.push_material(metal);

        Ok(())
    }

    /// Consumes a Plastic material
    fn consume_plastic(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        _modifier: &str,
        name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "5".to_string());
        let red = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let green = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let blue = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let specularity = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let roughness = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;

        self.modifiers.push(name.to_string());

        let colour = Spectrum([red, green, blue]);
        let plastic = if specularity < MIN_SPECULARITY {
            Material::Diffuse(Diffuse { colour })
        } else {
            Material::Plastic(Plastic {
                colour,
                specularity,
                roughness,
            })
        };

        scene.push_material(plastic);

        Ok(())
    }

    /// Consumes a Light material
    fn consume_light(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        _modifier: &str,
        name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "3".to_string());
        let red = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let green = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let blue = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;

        self.modifiers.push(name.to_string());

        let light = Material::Light(Light(Spectrum([red, green, blue])));
        scene.push_material(light);

        Ok(())
    }

    /// Consumes a Light material
    fn consume_mirror(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        _modifier: &str,
        name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "3".to_string());
        let red = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let green = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let blue = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;

        self.modifiers.push(name.to_string());

        let mirror = Material::Mirror(Mirror(Spectrum([red, green, blue])));
        scene.push_material(mirror);

        Ok(())
    }

    /// Consumes a Light material
    fn consume_dielectric(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        _modifier: &str,
        name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "5".to_string());
        let red = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let green = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let blue = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let refraction_index = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let _hartmans = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;

        self.modifiers.push(name.to_string());

        let dielectric = Material::Dielectric(Dielectric {
            colour: Spectrum([red, green, blue]),
            refraction_index,
        });
        scene.push_material(dielectric);

        Ok(())
    }

    /// Consumes a Light material
    fn consume_glass(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        _modifier: &str,
        name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        let mat = match t.as_bytes() {
            b"4" => {
                let red = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let green = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let blue = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let refraction_index = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let colour = Spectrum([red, green, blue]);
                Material::Glass(Glass {
                    colour,
                    refraction_index,
                })
            }
            b"3" => {
                let red = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let green = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let blue = self
                    .consume_token(source)?
                    .parse::<Float>()
                    .map_err(|e| e.to_string())?;
                let refraction_index = 1.52;
                let colour = Spectrum([red, green, blue]);
                Material::Glass(Glass {
                    colour,
                    refraction_index,
                })
            }
            _ => {
                self.error_here(format!(
                    "Incorrect Glass definition... expected 3 or 4 arguments; found '{}'",
                    t
                ));
            }
        };

        self.modifiers.push(name.to_string());
        scene.push_material(mat);

        Ok(())
    }

    /// Consumes a sphere
    fn consume_sphere(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        modifier: &str,
        _name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "4".to_string());
        let center_x = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let center_y = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let center_z = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let radius = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;

        let sphere = Sphere3D::new(radius, Point3D::new(center_x, center_y, center_z));

        let mod_index = self.get_modifier_index(modifier);
        scene.push_object(mod_index, mod_index, Primitive::Sphere(sphere));

        Ok(())
    }

    /// Consumes a sphere
    fn consume_source(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        modifier: &str,
        _name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "4".to_string());
        let dir_x = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let dir_y = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let dir_z = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?;
        let angle = self
            .consume_token(source)?
            .parse::<Float>()
            .map_err(|e| e.to_string())?
            .to_radians();
        let distant_source = DistantSource3D::new(Vector3D::new(dir_x, dir_y, dir_z), angle);

        let mod_index = self.get_modifier_index(modifier);
        scene.push_object(mod_index, mod_index, Primitive::Source(distant_source));
        Ok(())
    }

    /// Consumes a polygon
    fn consume_polygon(
        &mut self,
        source: &[u8],
        scene: &mut Scene,
        modifier: &str,
        _name: &str,
    ) -> Result<(), String> {
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let t = self.consume_token(source)?;
        assert_eq!(t, "0".to_string());
        let mut vertex_n = self
            .consume_token(source)?
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        if vertex_n % 3 != 0 {
            panic!("Incorrect Polygon... n%3 != 0")
        }

        let mut the_loop = Loop3D::new();

        while vertex_n > 0 {
            let x = self
                .consume_token(source)?
                .parse::<Float>()
                .map_err(|e| e.to_string())?;
            let y = self
                .consume_token(source)?
                .parse::<Float>()
                .map_err(|e| e.to_string())?;
            let z = self
                .consume_token(source)?
                .parse::<Float>()
                .map_err(|e| e.to_string())?;
            the_loop.push(Point3D::new(x, y, z))?;
            vertex_n -= 3;
        }
        let mod_index = self.get_modifier_index(modifier);

        the_loop.close()?;
        let polygon = Polygon3D::new(the_loop)?;
        let t: Triangulation3D = polygon.try_into()?;
        let triangles = t.get_trilist();

        for tri in triangles {
            scene.push_object(mod_index, mod_index, Primitive::Triangle(tri));
        }

        Ok(())
    }
}

impl Scene {
    /// Reads a Radiance file and builds a scene.
    pub fn from_radiance(filename: String) -> Result<Self, String> {
        let src = fs::read(filename).map_err(|e| e.to_string())?;
        Scene::from_radiance_source(&src)
    }

    /// Creates a scene from a slice of bytes read from a
    /// Radiance file
    pub fn from_radiance_source(source: &[u8]) -> Result<Self, String> {
        let mut ret = Self::default();

        let mut scanner = RadianceReader::default();

        while !scanner.is_done {
            scanner.consume_object(source, &mut ret)?;
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validate::assert_close;

    #[test]
    fn test_default() {
        let scanner = RadianceReader::default();
        assert!(!scanner.is_done);
        assert_eq!(scanner.current_char_index, 0);
    }

    #[test]
    #[should_panic(expected = "Error at line 0: This was a terrible error")]
    fn test_error_msg() {
        let scanner = RadianceReader::default();
        scanner.error_here("This was a terrible error".into());
    }

    #[test]
    fn test_consume_whitespace_empty() {
        let mut scanner = RadianceReader::default();
        let source = b"";
        assert!(!scanner.consume_whitespace(source));
        assert!(scanner.is_done);
    }

    #[test]
    fn test_consume_char_empty() {
        let mut scanner = RadianceReader::default();
        let source = b"";
        assert!(!scanner.consume_char(source));
        assert!(scanner.is_done);
    }

    #[test]
    fn test_token() -> Result<(), String> {
        let source = b"car with wheels";
        let mut scanner = RadianceReader::default();

        scanner.reach_next_token(source);
        assert_eq!(scanner.current_char_index, 0);
        assert_eq!(source[scanner.current_char_index], b'c');

        //===
        let source: &[u8] = "    car with wheels".as_bytes();
        let mut scanner = RadianceReader::default();

        scanner.reach_next_token(source);
        assert_eq!(scanner.current_char_index, 4);
        assert_eq!(source[scanner.current_char_index], b'c');

        //consume tokens
        let token_1 = scanner.consume_token(source)?;
        assert_eq!(token_1, "car".to_string());
        assert_eq!(source[scanner.current_char_index], b' ');
        assert_eq!(scanner.current_char_index, 7);

        assert_eq!("with".to_string(), scanner.consume_token(source)?);
        assert_eq!("wheels".to_string(), scanner.consume_token(source)?);

        let end = scanner.consume_token(source)?;
        assert_eq!("".to_string(), end);
        assert!(scanner.is_done);

        Ok(())
    }

    #[test]
    fn test_modifier_index() {
        let scanner = RadianceReader {
            modifiers: vec!["some_plastic".into()],
            ..RadianceReader::default()
        };
        assert_eq!(scanner.get_modifier_index("some_plastic".into()), 0);
    }

    #[test]
    #[should_panic]
    fn test_unknown_modifier_index() {
        let scanner = RadianceReader::default();
        scanner.get_modifier_index("some_plastic".into());
    }

    #[test]
    fn test_plastic() -> Result<(), String> {
        let src = b"void plastic red
        0
        0
        5 0.3 0.05 0.076 0.123 2.12312
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Plastic(m) = &scene.materials[0] {
            assert_close!(m.colour.0[0], 0.3);
            assert_close!(m.colour.0[1], 0.05);
            assert_close!(m.colour.0[2], 0.076);
            assert_close!(m.specularity, 0.123);
            assert_close!(m.roughness, 2.12312);
        } else {
            panic!("Not a plastic")
        }

        Ok(())
    }

    #[test]
    fn test_metal() -> Result<(), String> {
        let src = b"void metal red
        0
        0
        5 0.3 0.05 0.076 0.123 2.12312
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Metal(m) = &scene.materials[0] {
            assert_close!(m.colour.0[0], 0.3);
            assert_close!(m.colour.0[1], 0.05);
            assert_close!(m.colour.0[2], 0.076);
            assert_close!(m.specularity, 0.123);
            assert_close!(m.roughness, 2.12312);
        } else {
            panic!("Not a metal")
        }

        Ok(())
    }

    #[test]
    fn test_light() -> Result<(), String> {
        let src = b"void light red
        0
        0
        3 0.3 0.05 0.076
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Light(m) = &scene.materials[0] {
            assert_close!(m.0 .0[0], 0.3);
            assert_close!(m.0 .0[1], 0.05);
            assert_close!(m.0 .0[2], 0.076);
        } else {
            panic!("Not a metal")
        }

        Ok(())
    }

    #[test]
    fn test_mirror() -> Result<(), String> {
        let src = b"void mirror red
        0
        0
        3 0.3 0.05 0.076
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Mirror(m) = &scene.materials[0] {
            assert_close!(m.0 .0[0], 0.3);
            assert_close!(m.0 .0[1], 0.05);
            assert_close!(m.0 .0[2], 0.076);
        } else {
            panic!("Not a metal")
        }

        Ok(())
    }

    #[test]
    fn test_dielectric() -> Result<(), String> {
        let src = b"void dielectric red
        0
        0
        5 0.3 0.05 0.076 1.52 1.23
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Dielectric(m) = &scene.materials[0] {
            assert_close!(m.colour.0[0], 0.3);
            assert_close!(m.colour.0[1], 0.05);
            assert_close!(m.colour.0[2], 0.076);
            assert_close!(m.refraction_index, 1.52);
        } else {
            panic!("Not a metal")
        }

        Ok(())
    }

    #[test]
    fn test_glass_no_refraction() -> Result<(), String> {
        let src = b"void glass red
        0
        0
        3 0.3 0.05 0.076
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Glass(m) = &scene.materials[0] {
            assert_close!(m.colour.0[0], 0.3);
            assert_close!(m.colour.0[1], 0.05);
            assert_close!(m.colour.0[2], 0.076);
            assert_close!(m.refraction_index, 1.52);
        } else {
            panic!("Not a metal")
        }

        Ok(())
    }

    #[test]
    fn test_glass_refraction() -> Result<(), String> {
        let src = b"void glass red
        0
        0
        4 0.3 0.05 0.076 12.3
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?;
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert_eq!(scanner.modifiers[0], "red".to_string());
        assert_eq!(0, scanner.get_modifier_index(&"red".to_string()));
        if let Material::Glass(m) = &scene.materials[0] {
            assert_close!(m.colour.0[0], 0.3);
            assert_close!(m.colour.0[1], 0.05);
            assert_close!(m.colour.0[2], 0.076);
            assert_close!(m.refraction_index, 12.3);
        } else {
            panic!("Not a metal")
        }

        Ok(())
    }

    #[test]
    fn test_sphere() -> Result<(), String> {
        let src = b"void glass red
        0
        0
        4 0.3 0.05 0.076 12.3

        red sphere somesphere
        0
        0
        4 0.3 0.05 0.076 12.3
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?; // consume glass
        scanner.consume_object(src, &mut scene)?; // consume sphere
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);
        assert!(!scene.triangles.is_empty());

        assert_eq!(scene.normals.len(), scene.triangles.len());

        Ok(())
    }

    #[test]
    fn test_source() -> Result<(), String> {
        let src = b"void light red
        0
        0
        3 0.3 0.05 0.076

        red source up
        0
        0
        4 1. 2. 3. 4.
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?; // consume light
        scanner.consume_object(src, &mut scene)?; // consume source
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);

        assert!(scene.triangles.is_empty());

        assert_eq!(1, scene.distant_lights.len());

        assert!(scene.normals.is_empty());

        if let Primitive::Source(p) = &scene.distant_lights[0].primitive {
            let l = Vector3D::new(1., 2., 3.).get_normalized();
            assert_close!(p.direction.x, l.x);
            assert_close!(p.direction.y, l.y);
            assert_close!(p.direction.z, l.z);
            assert_close!(p.angle, 4. * crate::PI / 180.);
        } else {
            panic!("should have been a Sourvce")
        }

        Ok(())
    }

    #[test]
    fn test_polygon() -> Result<(), String> {
        let src = b"void glass red
        0
        0
        3 0.3 0.05 0.076

        red polygon pol
        0
        0
        9
            21. 12. 53.
            -4. 125. 66.
            75. 8.1 9.2
        ";

        let mut scene = Scene::new();
        let mut scanner = RadianceReader::default();
        scanner.consume_object(src, &mut scene)?; // consume light
        scanner.consume_object(src, &mut scene)?; // consume source
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scanner.modifiers.len(), 1);

        assert_eq!(scene.triangles.len(), 1);

        assert!(scene.distant_lights.is_empty());
        assert_eq!(scene.normals.len(), 1);

        let [ax, ay, az, bx, by, bz, cx, cy, cz] = [21., 12., 53., -4., 125., 66., 75., 8.1, 9.2];

        assert_close!(ax, scene.triangles[0][0]);
        assert_close!(ay, scene.triangles[0][1]);
        assert_close!(az, scene.triangles[0][2]);

        assert_close!(bx, scene.triangles[0][3]);
        assert_close!(by, scene.triangles[0][4]);
        assert_close!(bz, scene.triangles[0][5]);

        assert_close!(cx, scene.triangles[0][6]);
        assert_close!(cy, scene.triangles[0][7]);
        assert_close!(cz, scene.triangles[0][8]);

        Ok(())
    }
}
