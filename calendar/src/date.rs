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
use std::cmp::{Ordering, PartialOrd};
use std::fmt;
use std::ops::Sub;

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An extremely simple Date object. We don't
/// need anything else, I think.
/// It does not consider years at all!
/// Days and Months are counted from 1
/// (e.g. January is 1, not 0)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Date {
    /// Months of the year, from 1 to 12
    pub month: u8,

    /// Day of the month, from 1 to N
    pub day: u8,

    /// Hour of the day, from 0.0 to 23.999999
    pub hour: Float,
}

#[cfg(feature = "chrono")]
use chrono::{Datelike, NaiveDateTime, Timelike};

#[cfg(feature = "chrono")]
impl std::convert::From<NaiveDateTime> for Date {
    fn from(item: NaiveDateTime) -> Self {
        let month = item.month() as u8;
        let day = item.day() as u8;
        let hour_int = item.hour() as Float;
        let minute = item.minute() as Float;
        let seconds = item.second() as Float;
        let hour = hour_int + minute / 60. + seconds / 3600.;
        Self { month, day, hour }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // format hour

        let mut hour = self.hour.floor();

        let remainder = self.hour - hour;

        let mut minute = (remainder * 60.0).round();
        if minute == 60.0 {
            hour += 1.0;
            minute = 0.0
        }

        // ignore seconds

        write!(
            f,
            "{:02}/{:02} - {}:{:02}",
            self.month, self.day, hour, minute
        )
    }
}

impl Sub for Date {
    type Output = Float;

    fn sub(self, other: Self) -> Float {
        self.day_of_year() - other.day_of_year()
    }
}

impl Eq for Date {}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.month.cmp(&other.month) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                match self.day.cmp(&other.day) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => {
                        // Same day and month; compare by hour
                        if self.hour < other.hour {
                            Ordering::Less
                        } else if self.hour > other.hour {
                            Ordering::Greater
                        } else {
                            // They are the same
                            Ordering::Equal
                        }
                    }
                }
            }
        }
    }
}

impl Date {
    /// Transforms a Date into a `chrono` `NaiveDateTime`.
    ///
    /// Because `Date` does not have a year, we need to pass it as a parameter.        
    #[cfg(feature = "chrono")]
    pub fn into_naive_datetime(self, year: i32) -> NaiveDateTime {
        let hour = self.hour.floor();
        let remainder = self.hour - hour;
        let min = remainder * 60.0;
        let sec = (60.0 - min) * 60.0;

        let mut min = min.round() as u32;
        let mut sec = sec.round() as u32;

        if min == 60 {
            min = 59;
            sec = 59;
        }

        sec %= 60;

        NaiveDate::from_ymd_opt(year, self.month as u32, self.day as u32)
            .expect("Could not build chronos::Date")
            .and_hms_opt(hour.round() as u32, min, sec)
            .unwrap()
    }

    /// Interpolates between two dates
    pub fn interpolate(&self, other: Self, x: Float) -> Self {
        let n_self = self.day_of_year();
        let mut n_other = other.day_of_year();

        // If we celebrate new years between both dates
        if n_other < n_self {
            n_other += 365.
        }

        // Interpolate between the two
        let full_n = n_self + x * (n_other - n_self);
        Date::from_day_of_year(full_n % 365.)
    }

    /// Transforms a day of the year into a date
    pub fn from_day_of_year(n: Float) -> Self {
        if !(0. ..365.).contains(&n) {
            panic!("Impossible day of the year '{}' when building a date", n);
        }

        const CUMULATED_DAYS_BEFORE_MONTH: [Float; 12] = [
            0.,   // Jan
            31.,  // Feb
            59.,  // Mar
            90.,  // Apr
            120., // May
            151., // Jun
            181., // Jul
            212., // Aug
            243., // Sept
            273., // Oct
            304., // Nov
            334., // Dec
        ];

        for i in 0..12 {
            if CUMULATED_DAYS_BEFORE_MONTH[i] > n {
                let day_hour = n - CUMULATED_DAYS_BEFORE_MONTH[i - 1];
                let day = day_hour.floor();
                let hour = day_hour - day;

                debug_assert!(hour < 1.0);

                return Date {
                    month: i as u8,
                    day: 1 + day as u8,
                    hour: hour * 24.0,
                };
            }
        }
        // December
        let day_hour = n - CUMULATED_DAYS_BEFORE_MONTH[11];
        let day = day_hour.floor();
        let hour = day_hour - day;

        debug_assert!(hour < 1.0);

        // Return
        Date {
            month: 12,
            day: 1 + day as u8,
            hour: hour * 24.0,
        }
    }

    /// Retrieves the day of the year corresponding
    /// to the date (includes the decimals for the hour)
    pub fn day_of_year(&self) -> Float {
        const CUMULATED_DAYS_BEFORE_MONTH: [Float; 12] = [
            0.,   // Jan
            31.,  // Feb
            59.,  // Mar
            90.,  // Apr
            120., // May
            151., // Jun
            181., // Jul
            212., // Aug
            243., // Sept
            273., // Oct
            304., // Nov
            334., // Dec
        ];

        CUMULATED_DAYS_BEFORE_MONTH[self.month as usize - 1] + self.day as Float + self.hour / 24.0
            - 1.0
    }

    /// Adds a certain number of seconds to a date
    pub fn add_hours(&mut self, n_hours: Float) {
        // Calculate how many days are in those hours, and add them.
        let n_days = (n_hours / 24.).floor();
        self.add_days(n_days as usize);

        // Get remaining seconds and add them
        let remaining_hours = n_hours - 24. * n_days;

        self.hour += remaining_hours;
        if self.hour >= 24. {
            // ... remaining hours should be smaller than 24,
            // so adding a single day should be fine.
            self.add_days(1);
            self.hour %= 24.0;
        }
    }

    /// Adds a certain number of days to a date.
    pub fn add_days(&mut self, n_days: usize) {
        const N_DAYS_PER_MONTH: [u8; 12] = [
            31, // Jan
            28, // Feb
            31, // March
            30, // Apr
            31, // May
            30, // Jun
            31, // Jul
            31, // Aug
            30, // Sept
            31, // Oct
            30, // Nov
            31, // Dec
        ];

        // Lets recursively consume n_days month by month.
        let n_days = n_days % 365;
        if n_days > 0 {
            let would_be_day = self.day as usize + n_days; // Now we can add more than a year
            let n_days_this_month = N_DAYS_PER_MONTH[self.month as usize - 1];

            if would_be_day as u8 > n_days_this_month {
                // Add one month, considering that this might be a change in year
                self.month += 1;
                if self.month == 13 {
                    self.month = 1;
                }

                // reset the day
                self.day = 1;
                // And try again with the remaining days
                self.add_days(would_be_day - n_days_this_month as usize - 1);
            } else {
                // No change in month.
                self.day += n_days as u8;
            }
        }
    }

    /// Adds a timestep to the date.
    /// dt is in seconds
    pub fn add_seconds(&mut self, dt: Float) {
        self.add_hours(dt / 3600.);
    }

    /// Adds minutes to the date.    
    pub fn add_minutes(&mut self, minutes: Float) {
        self.add_hours(minutes / 60.);
    }

    /// Checks whether two dates have same day and month
    pub fn same_day(&self, other: Self) -> bool {
        self.month == other.month && self.day == other.day
    }

    /// Checks whether one date is earlier than another
    /// date. Dates are NOT earlier than themselvs.    
    #[deprecated = "Use 'self < other'  instead"]
    pub fn is_earlier(&self, other: Date) -> bool {
        if *self == other {
            return false;
        }
        self <= &other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate() {
        let start = Date {
            month: 1,
            day: 1,
            hour: 0.0,
        };

        assert!((start - start.interpolate(start, 0.0)).abs() < 1e-5);
        assert!((start - start.interpolate(start, 0.1)).abs() < 1e-5);
        assert!((start - start.interpolate(start, 0.5)).abs() < 1e-5);
        assert!((start - start.interpolate(start, 0.9)).abs() < 1e-5);

        let end = Date {
            month: 1,
            day: 2,
            hour: 23.999999999,
        };

        assert!(start.same_day(start.interpolate(end, 0.0)));
        assert!(start.same_day(start.interpolate(end, 0.2)));
        assert!(start.same_day(start.interpolate(end, 0.49)));
        assert!(
            start.same_day(start.interpolate(end, 0.49999)),
            "Start = {} | interpolated = {}",
            start,
            start.interpolate(end, 0.5)
        );
        assert!(end.same_day(start.interpolate(end, 0.51)));
        assert!(end.same_day(start.interpolate(end, 0.7)));
        assert!(end.same_day(start.interpolate(end, 0.9)));

        let start = Date {
            month: 12,
            day: 31,
            hour: 0.0,
        };

        let end = Date {
            month: 1,
            day: 1,
            hour: 23.9999999999999,
        };

        assert!(start.same_day(start.interpolate(end, 0.0)));
        assert!(start.same_day(start.interpolate(end, 0.2)));
        assert!(start.same_day(start.interpolate(end, 0.49)));
        assert!(end.same_day(start.interpolate(end, 0.5)));
        assert!(end.same_day(start.interpolate(end, 0.51)));
        assert!(end.same_day(start.interpolate(end, 0.7)));
        assert!(end.same_day(start.interpolate(end, 0.9)));

        let start = Date {
            month: 1,
            day: 31,
            hour: 0.0,
        };

        let end = Date {
            month: 2,
            day: 1,
            hour: 23.9999999999999,
        };

        assert!(start.same_day(start.interpolate(end, 0.0)));
        assert!(start.same_day(start.interpolate(end, 0.2)));
        assert!(start.same_day(start.interpolate(end, 0.49)));
        assert!(start.same_day(start.interpolate(end, 0.49999)));
        assert!(end.same_day(start.interpolate(end, 0.51)));
        assert!(end.same_day(start.interpolate(end, 0.7)));
        assert!(end.same_day(start.interpolate(end, 0.9)));
    }

    #[test]
    fn test_get_day_of_year() {
        let date = Date {
            month: 12,
            day: 31,
            hour: 12.0,
        };
        assert!((date - Date::from_day_of_year(364.5)).abs() < 0.01);

        let date = Date {
            day: 1,
            month: 1,
            hour: 0.0,
        };

        assert_eq!(0., date.day_of_year());
        assert!((date - Date::from_day_of_year(0.)).abs() < 0.01);

        let date = Date {
            day: 1,
            month: 1,
            hour: 12.0,
        };

        assert_eq!(0.5, date.day_of_year());
        assert!((date - Date::from_day_of_year(0.5)).abs() < 0.01);

        let date = Date {
            day: 4,
            month: 8,
            hour: 0.0,
        };

        assert_eq!(215., date.day_of_year());
        assert!((date - Date::from_day_of_year(215.)).abs() < 0.01);

        let date = Date {
            day: 4,
            month: 8,
            hour: 0.3 * 24.,
        };

        assert_eq!(215.3, date.day_of_year());
        assert!((date - Date::from_day_of_year(215.3)).abs() < 0.01);

        let date = Date {
            day: 24,
            month: 3,
            hour: 0.0,
        };

        assert_eq!(82., date.day_of_year());
        assert!((date - Date::from_day_of_year(82.)).abs() < 0.01);

        let date = Date {
            day: 24,
            month: 3,
            hour: 0.9 * 24.,
        };

        assert_eq!(82.9, date.day_of_year());
        assert!((date - Date::from_day_of_year(82.9)).abs() < 0.01);
    }

    fn test_compare(before: Date, after: Date) {
        assert!(after > before);
        assert!(!(before > after));

        assert!(before < after);
        assert!(!(after < before));

        assert!(before == before);
        assert!(after == after);
    }

    #[test]
    fn compare_equal() {
        // A single date... it is not later than itself
        let d = Date {
            month: 2,
            day: 3,
            hour: 1.,
        };
        assert!(!(d > d));
        assert!(!(d < d));
        assert!(d == d);
    }

    #[test]
    fn compare_by_month() {
        // month of difference
        let before = Date {
            month: 1,
            day: 1,
            hour: 0.,
        };
        let after = Date {
            month: 2,
            day: 1,
            hour: 0.,
        };

        test_compare(before, after);
    }

    #[test]
    fn compare_by_day() {
        // month of difference
        let before = Date {
            month: 2,
            day: 12,
            hour: 0.,
        };
        let after = Date {
            month: 2,
            day: 31,
            hour: 0.,
        };

        test_compare(before, after);
    }

    #[test]
    fn compare_by_hour() {
        // month of difference
        let before = Date {
            month: 2,
            day: 12,
            hour: 12.4,
        };
        let after = Date {
            month: 2,
            day: 12,
            hour: 21.,
        };

        test_compare(before, after);
    }

    #[test]
    fn compare_dates() {
        // month of difference
        let before = Date {
            month: 2,
            day: 12,
            hour: 12.4,
        };
        let after = Date {
            month: 9,
            day: 1,
            hour: 0.,
        };

        test_compare(before, after);
    }

    #[test]
    fn test_add_days_within_month() {
        let mut before = Date {
            month: 2,
            day: 12,
            hour: 12.4,
        };
        let after = Date {
            month: 2,
            day: 12,
            hour: 12.4,
        };

        before.add_days(0);
        assert!(before == after);

        // Add three days
        before.add_days(3);
        let after = Date {
            month: 2,
            day: 15,
            hour: 12.4,
        };
        assert!(before == after);
    }

    #[test]
    fn test_add_days_changing_month() {
        let mut before = Date {
            month: 1,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 2,
            day: 4,
            hour: 12.4,
        };

        // Add ten days
        before.add_days(10);
        assert!(before == after);

        // Change two months
        let mut before = Date {
            month: 1,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 3,
            day: 1,
            hour: 12.4,
        };

        // Add 35 days
        before.add_days(35);
        assert!(before == after);
    }

    #[test]
    fn test_add_days_changing_year() {
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 1,
            day: 4,
            hour: 12.4,
        };

        // Add ten days
        before.add_days(10);
        assert!(before == after);

        // Change two months
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 2,
            day: 2,
            hour: 12.4,
        };

        // Add 39 days
        before.add_days(39);
        assert!(before == after);

        // Add 365 days
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        before.add_days(365);
        assert!(before == after);

        // Add 365 + 39 days
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 2,
            day: 2,
            hour: 12.4,
        };
        before.add_days(365 + 39);
        assert!(before == after);
    }

    #[test]
    fn test_add_hours_seconds() {
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 12,
            day: 25,
            hour: 13.6,
        };

        // Add 1.2 hours
        before.add_hours(1.2);
        assert!(
            (before - after).abs() < 1e-5,
            "Before is {} | after is {}",
            before,
            after
        );

        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };

        // Add 1.2 hours, but in seconds
        before.add_seconds(4320.);
        assert!(
            (before - after).abs() < 1e-5,
            "Before is {} | after is {}",
            before,
            after
        );

        /* ADD DAYS, IN HOURS */
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 1,
            day: 4,
            hour: 12.4,
        };

        // Add ten days, in hours
        before.add_hours(10. * 24.);
        assert!(before == after);

        // Change two months
        let mut before = Date {
            month: 12,
            day: 25,
            hour: 12.4,
        };
        let after = Date {
            month: 2,
            day: 2,
            hour: 12.4,
        };

        // Add 39 days
        before.add_days(39);
        assert!(before == after);
    }

    #[test]
    fn test_sort() {
        let sorted = vec![
            Date {
                month: 1,
                day: 1,
                hour: 1.23,
            },
            Date {
                month: 1,
                day: 2,
                hour: 2.23,
            },
            Date {
                month: 2,
                day: 3,
                hour: 1.23,
            },
            Date {
                month: 5,
                day: 4,
                hour: 7.23,
            },
            Date {
                month: 7,
                day: 5,
                hour: 1.23,
            },
            Date {
                month: 12,
                day: 6,
                hour: 2.23,
            },
        ];

        let mut original = vec![
            Date {
                month: 5,
                day: 4,
                hour: 7.23,
            },
            Date {
                month: 12,
                day: 6,
                hour: 2.23,
            },
            Date {
                month: 1,
                day: 2,
                hour: 2.23,
            },
            Date {
                month: 2,
                day: 3,
                hour: 1.23,
            },
            Date {
                month: 7,
                day: 5,
                hour: 1.23,
            },
            Date {
                month: 1,
                day: 1,
                hour: 1.23,
            },
        ];

        original.sort();

        assert_eq!(sorted, original);
    }

    #[test]
    fn test_search() {
        let sorted = vec![
            Date {
                month: 1,
                day: 1,
                hour: 1.23,
            },
            Date {
                month: 1,
                day: 2,
                hour: 2.23,
            },
            Date {
                month: 2,
                day: 3,
                hour: 1.23,
            },
            Date {
                month: 5,
                day: 4,
                hour: 7.23,
            },
            Date {
                month: 7,
                day: 5,
                hour: 1.23,
            },
            Date {
                month: 12,
                day: 6,
                hour: 2.23,
            },
        ];

        if let Ok(i) = sorted.binary_search(&Date {
            month: 2,
            day: 3,
            hour: 1.23,
        }) {
            assert_eq!(i, 2)
        }

        if let Err(i) = sorted.binary_search(&Date {
            month: 2,
            day: 3,
            hour: 1.0,
        }) {
            assert_eq!(i, 2)
        }

        if let Err(i) = sorted.binary_search(&Date {
            month: 12,
            day: 13,
            hour: 1.0,
        }) {
            assert_eq!(i, 6)
        }

        // assert_eq!(sorted, original);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() -> Result<(), String> {
        use serde_json;

        let v = r#"{
            "month": 9,
            "day": 4, 
            "hour": 21
        }"#;
        let d: Date = serde_json::from_str(&v).map_err(|e| format!("{}", e))?;
        assert_eq!(d.month, 9);
        assert_eq!(d.day, 4);
        assert!((d.hour - 21.).abs() < 1e-5);
        Ok(())
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono() -> Result<(), String> {
        use chrono::NaiveDateTime;

        let v = "2014-11-28T21:00:09+09:00";
        let chrono_datetime =
            NaiveDateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%z").map_err(|e| e.to_string())?;

        let d: Date = chrono_datetime.into();
        assert_eq!(d.month, 11);
        assert_eq!(d.day, 28);
        assert!((d.hour - 21.0025).abs() < 1e-5, "hour is {}", d.hour);

        let v = "2023-06-30T23:59:59+12:00";
        let chrono_datetime =
            NaiveDateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%z").map_err(|e| e.to_string())?;

        let d: Date = chrono_datetime.into();
        assert_eq!(d.month, 06);
        assert_eq!(d.day, 30);
        println!("{d}");
        assert!((d.hour - 23.99972).abs() < 1e-5, "hour is {}", d.hour);
        Ok(())
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_loop() -> Result<(), String> {
        use crate::Period;

        // let start_str = "2022-12-25T00:00:00+12:00";
        // let chrono_start = NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%M:%S%z").map_err(|e| e.to_string())?;
        // let start : Date = chrono_start.into();

        // let end_str = "2022-12-31T23:59:59+12:00";
        // let chrono_end = NaiveDateTime::parse_from_str(&end_str, "%Y-%m-%dT%H:%M:%S%z").map_err(|e| e.to_string())?;
        // let end : Date = chrono_end.into();

        let start = Date {
            month: 12,
            day: 25,
            hour: 0.0,
        };
        let end = Date {
            month: 12,
            day: 31,
            hour: 23.999722222222225,
        };
        dbg!(end - start);
        let factory = Period::new(start, end, 3600.);
        for (i, d) in factory.enumerate() {
            dbg!(d, i);
        }

        Ok(())
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_into_naive_datetime() {
        use chrono::{Datelike, Timelike};

        let d = Date {
            month: 1,
            day: 1,
            hour: 23.9999,
        };

        let year = 2025;

        let out = d.into_naive_datetime(year);
        println!("{}", out);
        assert_eq!(out.year(), year);
        assert_eq!(out.month() as u8, d.month);
        assert_eq!(out.day() as u8, d.day);

        assert_eq!(out.hour(), 23);
        assert_eq!(out.minute(), 59);
        assert_eq!(out.second(), 59);
    }
}
