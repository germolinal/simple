/*
MIT License
Copyright (c) 2021 Germán Molina
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

#![deny(missing_docs)]

//! This is a library containing nn extremely simple Date object. The
//! purpose is to help perform Building Performance calculations, so it only
//! contains month, day and hour (in decimals). **It does not consider years at all**, 
//! days and Months are counted from 1 (e.g. January is 1, not 0). 
//! We don't need anything else, I think.
//! 
//! # Interaction with Serde
//! 
//! You can enable the `serde` feature and do stuff like this:
//! 
//! ```ignore
//! use calendar::Date;
//! use serde_json; // import "serde_json" and enable feature "serde"
//! 
//! let v = r#"{"month": 9,"day": 4, "hour": 21}"#;
//! let d : Date = serde_json::from_str(&v).unwrap();
//! assert_eq!(d.month, 9);
//! assert_eq!(d.day, 4);
//! assert!((d.hour - 21.).abs() < 1e-5);
//! ```
//! 
//! # Interaction with Chrono
//! 
//! You can enable the `chrono` feature and do stuff like this
//! 
//! ```ignore
//! use chrono::NaiveDateTime; // enable feature "chrono"
//! use calendar::Date;
//!        
//! let v = "2014-11-28T21:00:09+09:00";
//! let chrono_datetime  = NaiveDateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%z").unwrap();
//! 
//! let d : Date = chrono_datetime.into();
//! assert_eq!(d.month, 11);
//! assert_eq!(d.day, 28);
//! assert!((d.hour - 21.0025).abs() < 1e-5, "hour is {}", d.hour);
//! ```

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(not(feature = "float"))]
type Float = f64;


mod date;
pub use crate::date::Date;
mod date_factory;
pub use crate::date_factory::DateFactory;
