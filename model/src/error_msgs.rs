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

use std::fmt::Display;

/// Prints a warning message
pub(crate) fn print_warning_no_module<T: Display>(msg: T) {
    eprintln!("Warning: {}", msg)
}

/// Prints an error message
pub(crate) fn print_error_no_module<T: Display>(msg: T) {
    eprintln!("Error: {}", msg)
}

/// Prints a warning message.
///
/// This is meant to be used from within the simulation modules,
/// so that the user can know where is the warning comming from
///
/// ```
/// use model::print_warning;
/// print_warning("Name of the module", "some warning");
/// ```
pub fn print_warning<C: Display, T: Display>(module_name: C, msg: T) {
    print_warning_no_module(format!("[in {}] {}", module_name, msg))
}

/// Prints an error message.
///
/// This is meant to be used from within the simulation modules,
/// so that the user can know where is the error comming from
///
/// ```
/// use model::print_error;
/// print_error("Name of the module", "some warning");
/// ```
pub fn print_error<C: Display, T: Display>(module_name: C, msg: T) {
    print_error_no_module(format!("[in {}] {}", module_name, msg))
}
