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
use crate::Float;
use crate::date::Date;

#[cfg(feature="serde")]
use serde::{Serialize, Deserialize};

/// a struct that will give us dates    
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
pub struct DateFactory {

    /// End date of the iterator
    end: Date,

    /// Current date in the iterator
    current: Date,

    /// The timestep of the iterator, in seconds
    dt: Float
}

impl DateFactory {

    /// Creates a new Factory starting on `start` and ending in `end`, 
    /// separated by `dt` seconds each time.
    pub fn new( start: Date, end: Date, dt: Float)-> Self{
        Self{            
            end,
            current: start,
            dt,
        }
    }
}

impl Iterator for DateFactory {
    type Item = Date;

    fn next(&mut self)->Option<Self::Item>{
        
        let mut new = self.current;
        new.add_seconds(self.dt);
        let full_loop = new <= self.current ;
        let finished = new >= self.end;
        if finished || full_loop {
            return None
        }
        self.current = new;
        Some(self.current)

    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterate_full_year(){

        for n in 1..7 {

            let start = Date{
                month: 1, day: 1, hour: 0.0
            };
            let end = Date {
                month: 12, day: 31, hour: 24.0
            };            
            let dt = 60.*60. / (n as Float);
            let dates = DateFactory::new(start, end, dt);        
    
            let mut count = 1;
            for _ in dates {            
                // assert_eq!(count,d.hour);
                count += 1;
            }
            assert!((count as i32 - 8760 as i32 * n as i32).abs() <= 1);        
        }

    }

}
