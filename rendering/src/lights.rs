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

use crate::Float;
use crate::sampleable_trait::Sampleable;
use crate::colour::Spectrum;

// Flags
const DELTA_POSITION_LIGHT: u8 = 1;
const DELTA_DIRECTION_LIGHT : u8 = 2;
const AREA_LIGHT : u8 = 4;
const INFINITE_LIGHT : u8 = 8;

trait Light : Sampleable {    
    
    /// Returns the light flags associated with the 
    /// light source
    fn flags(&self)->u8;


    /// Returns the number of samples that should be used for this
    /// specific light source
    fn n_samples(&self)->usize;

    /// Checks whether a light source has a Dirac's delta position
    /// or direction
    fn is_delta_light(&self)->bool{        
        self.flags() & DELTA_POSITION_LIGHT == 1 ||
        self.flags() & DELTA_DIRECTION_LIGHT == 1        
    }

    // fn power(&self)->Float;
    
    
    fn sample_li(&self)->Spectrum;
    fn pdf_li(&self)->Float;

    fn le(&self)->Spectrum;
    fn sample_le(&self)->Spectrum;
    fn pdf_le(&self)->Float;
}




