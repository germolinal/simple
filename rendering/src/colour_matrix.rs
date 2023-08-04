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
use crate::colour::Spectrum;
use crate::Float;
use matrix::{GenericMatrix, Matrix};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub type ColourMatrix = GenericMatrix<Spectrum>;

pub fn average_matrix(dc: &Matrix) -> Matrix {
    let (nrows, _ncols) = dc.size();
    let average_operator = Matrix::new(1. / nrows as Float, 1, nrows);
    // return
    &average_operator * dc
}

pub fn colour_matrix_to_radiance(cm: &ColourMatrix) -> Matrix {
    let (nrows, ncols) = cm.size();
    let mut ret = Matrix::new(0.0, nrows, ncols);
    for row in 0..nrows {
        for col in 0..ncols {
            let color = cm.get(row, col).unwrap();
            ret.set(row, col, color.radiance()).unwrap();
        }
    }
    ret
}

pub fn colour_matrix_to_luminance(cm: &ColourMatrix) -> Matrix {
    let (nrows, ncols) = cm.size();
    let mut ret = Matrix::new(0.0, nrows, ncols);
    for row in 0..nrows {
        for col in 0..ncols {
            let color = cm.get(row, col).unwrap();
            ret.set(row, col, color.luminance()).unwrap();
        }
    }
    ret
}

pub fn save_colour_matrix(cm: &ColourMatrix, filename: &Path) -> Result<(), String> {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => return Err(format!("{:?}", e)),
    };

    // Header
    let (nrows, ncols) = cm.size();
    writeln!(&mut file, "#?SIMPLE").unwrap();
    writeln!(&mut file, "NROWS={}", nrows).unwrap();
    writeln!(&mut file, "NCOLS={}", ncols).unwrap();
    writeln!(&mut file, "NCOMP=3").unwrap();
    writeln!(&mut file, "FORMAT=ascii").unwrap();
    writeln!(&mut file).unwrap();

    // Body
    for r in 0..nrows {
        for c in 0..ncols {
            let v = cm.get(r, c).unwrap();
            write!(&mut file, "{}\t", v).unwrap();
        }
        writeln!(&mut file).unwrap();
    }
    // return
    Ok(())
}

pub fn save_matrix(cm: &Matrix, filename: &Path) -> Result<(), String> {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => return Err(format!("{:?}", e)),
    };

    // Header
    let (nrows, ncols) = cm.size();
    writeln!(&mut file, "#?SIMPLE").unwrap();
    writeln!(&mut file, "NROWS={}", nrows).unwrap();
    writeln!(&mut file, "NCOLS={}", ncols).unwrap();
    writeln!(&mut file, "NCOMP=1").unwrap();
    writeln!(&mut file, "FORMAT=ascii").unwrap();
    writeln!(&mut file).unwrap();

    // Body
    for r in 0..nrows {
        for c in 0..ncols {
            let v = cm.get(r, c).unwrap();
            write!(&mut file, "{}\t", v).unwrap();
        }
        writeln!(&mut file).unwrap();
    }
    // return
    Ok(())
}

pub fn read_colour_matrix(filename: &Path) -> Result<ColourMatrix, String> {
    let content = match std::fs::read_to_string(filename) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Could not read Matrix file '{}'",
                filename.to_str().unwrap()
            ))
        }
    };
    let filename = filename.to_str().unwrap();

    // Read header
    let mut nrows: Option<usize> = None;
    let mut ncols: Option<usize> = None;
    let mut header_lines = 0;
    for line in content.lines() {
        header_lines += 1;
        // If we reach a blank line, we are over with the header.
        if line.is_empty() || line.as_bytes()[0].is_ascii_whitespace() {
            break;
        }

        if line.starts_with("NROWS") {
            let tuple: Vec<&str> = line.split('=').collect();
            if tuple.len() != 2 {
                return Err(format!(
                    "Expecting NROWS line to be in the format 'NROWS=number'... found '{}'",
                    line
                ));
            }
            nrows = match tuple[1].parse::<usize>() {
                Ok(v) => Some(v),
                Err(_) => {
                    return Err(format!("Expecting NROWS line to be in the format 'NROWS=number', but did not find a number... found '{}'", tuple[1]));
                }
            };
            continue;
        }
        if line.starts_with("NCOLS") {
            let tuple: Vec<&str> = line.split('=').collect();
            if tuple.len() != 2 {
                return Err(format!(
                    "Expecting NCOLS line to be in the format 'NCOLS=number'... found '{}'",
                    line
                ));
            }
            ncols = match tuple[1].parse::<usize>() {
                Ok(v) => Some(v),
                Err(_) => {
                    return Err(format!("Expecting NCOLS line to be in the format 'NCOLS=number', but did not find a number... found '{}'", tuple[1]));
                }
            };
            continue;
        }
        if line.starts_with("NCOMP") {
            let tuple: Vec<&str> = line.split('=').collect();
            if tuple.len() != 2 {
                return Err(format!(
                    "Expecting NCOMP line to be in the format 'NCOMP=number'... found '{}'",
                    line
                ));
            }
            let ncomp = match tuple[1].parse::<usize>() {
                Ok(fvalue) => fvalue,
                Err(_) => {
                    return Err(format!("Expecting NCOMP line to be in the format 'NCOMP=number', but did not find a number... found '{}'", tuple[1]));
                }
            };
            if ncomp != 3 {
                return Err(format!(
                    "Expecting 3 components in Colour Matrix... found {}",
                    ncomp
                ));
            }
            continue;
        }
    }

    // Check that the header info was fine
    if nrows.is_none() {
        return Err(format!(
            "Matrix in file '{}' does not include number of rows in header",
            filename
        ));
    }
    if ncols.is_none() {
        return Err(format!(
            "Matrix in file '{}' does not include number of columns in header",
            filename
        ));
    }
    let nrows = nrows.unwrap();
    let ncols = ncols.unwrap();
    let mut matrix = ColourMatrix::new(Spectrum::BLACK, nrows, ncols);

    // Read content.
    for (nrow, line) in content.lines().skip(header_lines).enumerate() {
        let ln = nrow + header_lines;
        let values: Vec<&str> = line.split_ascii_whitespace().collect();
        if values.len() != 3 * ncols {
            return Err(format!(
                "Expecting {} values in line {}... found {}",
                3 * ncols,
                ln,
                values.len()
            ));
        }
        let mut ncol = 0;
        while ncol < ncols {
            let red = match values[3 * ncol].parse::<Float>() {
                Ok(fvalue) => fvalue,
                Err(_) => {
                    return Err(format!(
                        "Incorrectly formated line {} in matrix in file '{}'",
                        ln, filename
                    ));
                }
            };
            let green = match values[3 * ncol + 1].parse::<Float>() {
                Ok(fvalue) => fvalue,
                Err(_) => {
                    return Err(format!(
                        "Incorrectly formated line {} in matrix in file '{}'",
                        ln, filename
                    ));
                }
            };
            let blue = match values[3 * ncol + 2].parse::<Float>() {
                Ok(fvalue) => fvalue,
                Err(_) => {
                    return Err(format!(
                        "Incorrectly formated line {} in matrix in file '{}'",
                        ln, filename
                    ));
                }
            };

            matrix
                .set(nrow, ncol, Spectrum([red, green, blue]))
                .unwrap();

            ncol += 1;
        }
    }

    // return
    Ok(matrix)
}

pub fn read_matrix(filename: &Path) -> Result<Matrix, String> {
    let content = match std::fs::read_to_string(filename) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Could not read Matrix file '{}'",
                filename.to_str().unwrap()
            ))
        }
    };
    if content.is_empty() {
        return Err(format!("file '{}' is empty", filename.to_str().unwrap()));
    }
    // Read header
    let filename = filename.to_str().unwrap();
    let mut nrows: Option<usize> = None;
    let mut ncols: Option<usize> = None;
    let mut header_lines = 0;
    for line in content.lines() {
        header_lines += 1;
        // If we reach a blank line, we are over with the header.
        if line.is_empty() || line.as_bytes()[0].is_ascii_whitespace() {
            break;
        }

        if line.starts_with("NROWS") {
            let tuple: Vec<&str> = line.split('=').collect();
            if tuple.len() != 2 {
                return Err(format!(
                    "Expecting NROWS line to be in the format 'NROWS=number'... found '{}'",
                    line
                ));
            }
            nrows = match tuple[1].parse::<usize>() {
                Ok(v) => Some(v),
                Err(_) => {
                    return Err(format!("Expecting NROWS line to be in the format 'NROWS=number', but did not find a number... found '{}'", tuple[1]));
                }
            };
            continue;
        }
        if line.starts_with("NCOLS") {
            let tuple: Vec<&str> = line.split('=').collect();
            if tuple.len() != 2 {
                return Err(format!(
                    "Expecting NCOLS line to be in the format 'NCOLS=number'... found '{}'",
                    line
                ));
            }
            ncols = match tuple[1].parse::<usize>() {
                Ok(v) => Some(v),
                Err(_) => {
                    return Err(format!("Expecting NCOLS line to be in the format 'NCOLS=number', but did not find a number... found '{}'", tuple[1]));
                }
            };
            continue;
        }
        if line.starts_with("NCOMP") {
            let tuple: Vec<&str> = line.split('=').collect();
            if tuple.len() != 2 {
                return Err(format!(
                    "Expecting NCOMP line to be in the format 'NCOMP=number'... found '{}'",
                    line
                ));
            }
            let ncomp = match tuple[1].parse::<usize>() {
                Ok(fvalue) => fvalue,
                Err(_) => {
                    return Err(format!("Expecting NCOMP line to be in the format 'NCOMP=number', but did not find a number... found '{}'", tuple[1]));
                }
            };
            if ncomp != 1 {
                return Err(format!(
                    "Expecting 1 components in Matrix... found {}",
                    ncomp
                ));
            }
            continue;
        }
    }

    // Check that the header info was fine
    if nrows.is_none() {
        return Err(format!(
            "Matrix in file '{}' does not include number of rows in header",
            filename
        ));
    }
    if ncols.is_none() {
        return Err(format!(
            "Matrix in file '{}' does not include number of columns in header",
            filename
        ));
    }
    let nrows = nrows.unwrap();
    let ncols = ncols.unwrap();
    let mut matrix = Matrix::new(0.0, nrows, ncols);

    // Read content.
    for (nrow, line) in content.lines().skip(header_lines).enumerate() {
        let ln = nrow + header_lines;
        let values: Vec<&str> = line.split_ascii_whitespace().collect();
        if values.len() != ncols {
            return Err(format!(
                "Expecting {} values in line {}... found {}",
                ncols,
                ln,
                values.len()
            ));
        }
        let mut ncol = 0;
        while ncol < ncols {
            let v = match values[ncol].parse::<Float>() {
                Ok(fvalue) => fvalue,
                Err(_) => {
                    return Err(format!(
                        "Incorrectly formated line {} in matrix in file '{}'",
                        ln, filename
                    ));
                }
            };
            matrix.set(nrow, ncol, v).unwrap();

            ncol += 1;
        }
    }

    // return
    Ok(matrix)
}
