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

use std::{collections::HashMap, fmt::Display};

use crate::{Model, SimulationStateHeader};

/// The scanner
pub(crate) struct SimpleScanner<'a> {
    /// The line being read (initialized as 0 by default)
    line: usize,

    source: &'a [u8],

    current_index: usize,

    start_index: usize,

    finished: bool,
}

impl<'a> SimpleScanner<'a> {
    /// Creates a new [`SimpleScanner`]
    pub(crate) fn new(source: &'a [u8], line: usize) -> Self {
        Self {
            finished: source.is_empty(),
            source,
            line,
            current_index: 0,
            start_index: 0,
        }
    }

    /// Creates an syntax error and sets the `any_errors` flag in the scanner to `true`
    fn make_error_msg<S: Display>(msg: S, ln: usize) -> String {
        format!("Error [in line {}]: {}", ln, msg)
    }

    /// Advances one `char` in the `source`, returning the consumed
    /// `char` inside of an `Option`. If finished, it will mark the
    /// [`SimpleScanner`] as finished and return `None`
    fn advance(&mut self) -> Option<char> {
        if let Some(v) = self.source.get(self.current_index) {
            self.current_index += 1;
            if self.current_index == self.source.len() {
                self.finished = true;
            }
            Some(*v as char)
        } else {
            self.finished = true;
            None
        }
    }

    /// Gets the `char` at the `current_index`. Returns `\0` if
    /// finished.
    fn peek(&self) -> char {
        if self.finished {
            return '\0';
        }
        self.source[self.current_index] as char
    }

    /// Gets the `char` following the `current_index`. Returns `\0` if
    /// finished.
    fn peek_next(&self) -> char {
        if self.finished || self.current_index + 1 == self.source.len() {
            return '\0';
        }

        self.source[self.current_index + 1] as char // .clone().add(1) as char;
    }

    /// Skips the white spaces and the comments and all
    /// those things.
    fn skip_white_space(&mut self) -> Result<(), String> {
        // println!("---> '{}'", self.peek());
        // Prevent segfault
        if self.finished {
            return Ok(());
        }

        loop {
            match self.peek() {
                ' ' => {
                    self.advance().ok_or("Unexpected EOF when scanning")?;
                }
                '\r' => {
                    self.advance().ok_or("Unexpected EOF when scanning")?;
                }
                '\t' => {
                    self.advance().ok_or("Unexpected EOF when scanning")?;
                }
                '\n' => {
                    self.line += 1;
                    self.advance().ok_or("Unexpected EOF when scanning")?;
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // Single line comment
                        while self.peek() != '\n' && !self.finished {
                            self.advance().ok_or("Unexpected EOF when scanning")?;
                        }
                    } else if self.peek_next() == '*' {
                        // Consume slash and star
                        self.advance().ok_or("Unexpected EOF when scanning")?;
                        self.advance().ok_or("Unexpected EOF when scanning")?;
                        // Block comment
                        loop {
                            // Check if it is end
                            if self.finished {
                                return Ok(());
                            }

                            // Check if end of block comment
                            if self.peek() == '*' && self.peek_next() == '/' {
                                // Consume slash and star
                                self.advance().ok_or("Unexpected EOF when scanning")?;
                                self.advance().ok_or("Unexpected EOF when scanning")?;
                                break; // get out of the block comment loop
                            }
                            if let '\n' = self.advance().ok_or("Unexpected EOF when scanning")? {
                                self.line += 1;
                            }
                        }
                    } else {
                        return Ok(());
                    }
                }
                _ => return Ok(()),
            };
        }
    }

    /// Consumes a whole Identifier
    fn identifier(&mut self) -> Result<(usize, usize), String> {
        // scan the whole thing, until we find something that
        // is not a number, letter or '_'
        let mut c = self.peek();
        while c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' {
            match self.advance() {
                Some(_) => c = self.peek(),
                None => {
                    let errmsg = Self::make_error_msg("Unexpected End of File", self.line);
                    return Err(errmsg);
                }
            }
        }

        Ok((self.start_index, self.current_index))
    }

    /// Consumes an object and returns the start and end of that object.
    fn object(&mut self) -> Result<(usize, usize), String> {
        let mut levels = 0;
        let mut started = false;

        let (mut open, mut close) = ('{', '}');

        while levels > 0 || !started {
            let next = self.peek();
            if !started && (next == ',') {
                let errmsg = Self::make_error_msg("Malformed enum object. These should have parentheses (e.g., not 'ShelterClass::Urban' but 'ShelterClass::Urban()' )", self.line);
                return Err(errmsg);
            }
            if !started && next == '(' {
                // We are opening with this.
                open = '(';
                close = ')';
            }

            if next == open {
                levels += 1;
                started = true;
            }
            if next == close {
                levels -= 1;
            }
            if next == '\n' {
                self.line += 1;
            }
            self.current_index += 1;

            if self.current_index == self.source.len() {
                self.finished = true;
                break;
            }
        }

        // return
        Ok((self.start_index, self.current_index))
    }

    /// Updates the start index; i.e., sets the `start_index` to the `current_index`
    fn update_start_index(&mut self) {
        self.start_index = self.current_index;
    }

    /// Parses a whole [`Model`] from a text file
    pub(crate) fn parse_model(&mut self) -> Result<(Model, SimulationStateHeader), String> {
        let mut data = HashMap::<String, Vec<(&str, usize)>>::new();

        loop {
            self.skip_white_space()?;
            self.update_start_index();

            if self.finished {
                break;
            }

            // Scan identifier
            let (ini, fin) = self.identifier()?;
            let ident = &self.source[ini..fin];

            // Skip whitespaces
            self.skip_white_space()?;
            self.update_start_index();

            // Scan Object
            let (ini, fin) = self.object()?;
            let obj = &self.source[ini..fin];

            // Make it a string
            let obj_str = match std::str::from_utf8(obj) {
                Ok(v) => v,
                Err(e) => return Err(format!("{}", e)),
            };

            // Store.
            let key = std::str::from_utf8(ident)
                .expect("Could not scan")
                .to_string();
            if let Some(v) = data.get_mut(&key) {
                v.push((obj_str, self.line));
            } else {
                data.insert(key, vec![(obj_str, self.line)]);
            }
        }
        // Now, build the model
        let mut model = Model::default();
        let read_order = vec![
            // Order matters with these ones
            "Substance",
            "Material",
            "Construction",
            "Surface",
            "Fenestration",
            "Space",
            // These are independent
            "Building",
            "HVAC",
            "Luminaire",
            "Object",
            "Output",
            "SiteDetails",
            "SolarOptions",
        ];

        for ident in read_order {
            if data.get(ident).is_none() {
                continue;
            }

            for (obj_str, ln) in data.get(ident).unwrap().iter() {
                match ident.as_bytes() {
                    b"Building" => {
                        let s: crate::Building = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_building(s);
                    }
                    b"Construction" => {
                        let s: crate::Construction = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_construction(s);
                    }
                    b"Fenestration" => {
                        let s: crate::Fenestration = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_fenestration(s)?;
                    }
                    b"HVAC" => {
                        let s: crate::HVAC = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_hvac(s)?;
                    }
                    b"Luminaire" => {
                        let s: crate::Luminaire = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_luminaire(s)?;
                    }
                    b"Material" => {
                        let s: crate::Material = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_material(s);
                    }
                    b"Object" => {
                        let s: crate::Object = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.objects.push(s);
                    }
                    b"Output" => {
                        let s: crate::Output = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.outputs.push(s);
                    }
                    b"SiteDetails" => {
                        let s: crate::SiteDetails = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.site_details = Some(s);
                    }
                    b"SolarOptions" => {
                        let s: crate::SolarOptions = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.solar_options = Some(s);
                    }
                    b"Space" => {
                        let s: crate::Space = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_space(s);
                    }
                    b"Surface" => {
                        let s: crate::Surface = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_surface(s);
                    }
                    b"Substance" => {
                        let s: crate::Substance = match json5::from_str(obj_str) {
                            Ok(s) => s,
                            Err(e) => {
                                let errmsg = Self::make_error_msg(format!("{}", e), *ln);
                                return Err(errmsg);
                            }
                        };
                        model.add_substance(s);
                    }
                    _ => {
                        let errmsg = Self::make_error_msg(
                            format!("unknown identifier {}", ident),
                            self.line,
                        );
                        return Err(errmsg);
                    }
                }
            }
        }

        let state = model.take_state().ok_or("Could not take state")?;
        Ok((model, state))
    }
}

#[cfg(test)]
mod testing {

    use super::*;

    #[test]
    fn test_error() {
        let err = SimpleScanner::make_error_msg("the error", 1);
        assert_eq!(err, "Error [in line 1]: the error")
    }

    #[test]
    fn scan() -> Result<(), String> {
        let source = b"SomeObject { data data }";
        let mut scan = SimpleScanner::new(source, 1);

        assert_eq!(scan.line, 1);
        assert_eq!(scan.source[scan.start_index], source[scan.start_index]);
        assert_eq!(scan.source[scan.current_index], source[scan.current_index]);

        scan.skip_white_space()?;
        let (ini, fin) = scan.identifier()?;
        let ident = &source[ini..fin];
        assert_eq!(ident, b"SomeObject");

        scan.skip_white_space()?;
        scan.update_start_index();

        let (ini, fin) = scan.object()?;
        let object = &source[ini..fin];
        let obj_str = std::str::from_utf8(&object).map_err(|e| e.to_string())?;
        println!("'{}'", obj_str);
        assert_eq!(object, b"{ data data }");
        Ok(())
    }
}
