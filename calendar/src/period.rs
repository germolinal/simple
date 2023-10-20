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
use crate::date::Date;
use crate::Float;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// a struct that will give us dates    
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Period {
    /// End date of the iterator
    end: Date,

    /// Start date of the iterator
    start: Date,

    /// Current date in the iterator
    current: Date,

    /// The timestep of the iterator, in seconds
    dt: Float,

    /// If the start is later than the end, then it is a loop
    /// that goes through new years.
    goes_through_new_year: bool,
}

impl Period {
    /// Creates a new Factory starting on `start` and ending in `end`,
    /// separated by `dt` seconds each time.
    pub fn new(start: Date, end: Date, dt: Float) -> Self {
        if start.hour >= 24.0 {
            panic!(
                "Wrong hour for start: found {}, but hours go from 0 to 23.9999...",
                start.hour
            )
        }
        if end.hour >= 24.0 {
            panic!(
                "Wrong hour for end: found {}, but hours go from 0 to 23.9999...",
                end.hour
            )
        }

        let current = start;
        Self {
            end,
            start,
            current,
            dt,
            goes_through_new_year: start > end,
        }
    }

    /// Checks if a date (year-agnostic) is contained
    ///
    /// ```
    /// use calendar::{Date, Period};
    ///
    /// let start = Date{
    ///     month: 1, day: 2, hour: 1.23
    /// };
    ///
    /// let end = Date{
    ///     month: 1, day: 3, hour: 1.23
    /// };
    /// let period = Period::new(start,end, 3600.);
    ///
    /// assert!(period.contains(Date{
    ///     month: 1, day: 2, hour: 5.0
    /// }));
    ///
    /// // What about a period that loops through the end of the year?
    ///
    /// let start = Date{
    ///     month: 12, day: 2, hour: 1.23
    /// };
    ///
    /// let end = Date{
    ///     month: 1, day: 3, hour: 1.23
    /// };
    /// let period = Period::new(start,end, 3600.);
    ///
    /// assert!(period.contains(Date{
    ///     month: 12, day: 5, hour: 5.0
    /// }));
    ///
    ///
    /// ```
    pub fn contains(&self, date: Date) -> bool {
        if !self.goes_through_new_year {
            // (jan 1) ---- Start --------X------- End ---- (dec 31)
            self.end >= date && self.start <= date
        } else {
            // (jan 1) -X-- End ----------------- Start --X- (dec 31)
            date <= self.end || date >= self.start
        }
    }
}

impl Iterator for Period {
    type Item = Date;

    fn next(&mut self) -> Option<Self::Item> {
        let old = self.current;

        let mut new = self.current;

        let vs_end_before = new <= self.end;
        new.add_seconds(self.dt);
        let vs_end_after = new <= self.end;
        let crossed_end = vs_end_before != vs_end_after;
        let crossed_ny = new < old;

        // println!("     ---- old = {} | new = {} | end = {} |||| vs_end_before = {} | vs_end_after = {} | crossed_new? = {}", old, new, self.end, vs_end_before, vs_end_after, crossed_ny);

        // if !self.goes_through_new_year && (crossed_end || crossed_ny)
        //     || self.goes_through_new_year && crossed_end && !crossed_ny
        if (crossed_ny || crossed_end) && (!crossed_ny || !self.goes_through_new_year) {
            return None;
        }
        self.current = new;
        // Some(old)
        Some(self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let start = Date {
            month: 1,
            day: 1,
            hour: 12.,
        };
        let end = Date {
            month: 1,
            day: 2,
            hour: 12.,
        };
        assert!(!Period::new(start, end, 0.1).goes_through_new_year);

        let start = Date {
            month: 12,
            day: 1,
            hour: 12.,
        };
        let end = Date {
            month: 1,
            day: 2,
            hour: 12.,
        };
        assert!(Period::new(start, end, 0.1).goes_through_new_year);

        let start = Date {
            month: 1,
            day: 1,
            hour: 12.,
        };
        let end = Date {
            month: 1,
            day: 1,
            hour: 12.,
        };
        assert!(!Period::new(start, end, 0.1).goes_through_new_year);
    }

    #[test]
    fn test_iterate_full_year() {
        for n in 1..7 {
            let start = Date {
                month: 1,
                day: 1,
                hour: 0.0,
            };
            let end = Date {
                month: 12,
                day: 31,
                hour: 23.9,
            };
            let dt = 60. * 60. / (n as Float);
            let dates = Period::new(start, end, dt);

            let mut count = 1;
            for _d in dates {
                // println!("{}, {}, {}", _d, dates.current, dates.end);
                // assert_eq!(count,d.hour);
                assert!(count < 8760 * 10, "Count is {} | n = {}", count, n);
                count += 1;
            }
            let aux = (count as i32 - 8760 as i32 * n as i32).abs();
            assert!(
                aux == 0,
                "... found {} (count = {} | n = {})",
                aux,
                count,
                n
            );
            break;
        }
    }

    #[test]
    fn test_iterate_through_newyears() {
        let start = Date {
            month: 12,
            day: 31,
            hour: 0.0,
        };
        let end = Date {
            month: 1,
            day: 2,
            hour: 23.99999,
        };

        let f = Period::new(start, end, 1100.);

        let mut count = 0;

        for d in f.into_iter() {
            // assert!(count < 48);
            count += 1;
            println!("{}", d)
        }
        dbg!(count);
    }

    #[test]
    fn test_warmup_period() {
        let warmup_period = Period {
            end: Date {
                month: 1,
                day: 1,
                hour: 1.0,
            },
            start: Date {
                month: 12,
                day: 25,
                hour: 1.0,
            },
            current: Date {
                month: 12,
                day: 25,
                hour: 1.0,
            },
            dt: 3600.0,
            goes_through_new_year: true,
        };

        for d in warmup_period {
            println!("{}", d)
        }
    }
}
