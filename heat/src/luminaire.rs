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

use model::{Luminaire, Model};
use std::sync::Arc;

/// An HVAC element from the point of view of the thermal
/// model.
pub struct ThermalLuminaire {
    /// The parent Luminaire
    pub(crate) parent: Luminaire,

    /// The space whwre the luminaire is located
    pub(crate) target_space_index: usize,
}

impl ThermalLuminaire {
    /// Builds a new [`ThermalLuminaire`] from an [`Luminaire`] and its location
    pub fn from(lum: &Arc<Luminaire>, model: &Model) -> Result<Self, String> {
        let parent = (**lum).clone();
        for (i, s) in model.spaces.iter().enumerate() {
            if s.name() == parent.target_space()? {
                return Ok(Self {
                    parent,
                    target_space_index: i,
                });
            }
        }
        Err(format!(
            "Luminaire is supposed to be in a space called '{}'... but it was not found",
            parent.target_space()?
        ))
    }
}
