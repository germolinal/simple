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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A day of the week
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DayOfWeek {
    /// Monday
    Monday,
    /// Tuesday
    Tuesday,
    /// Wednesday
    Wednesday,
    /// Thursday
    Thursday,
    /// Friday
    Friday,
    /// Saturday
    Saturday,
    /// Sunday
    Sunday,
}

impl DayOfWeek {
    const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// Parses the name of a day (e.g., `"Sunday"`, as in EPW files),
    /// ignoring case and surrounding spaces
    ///
    /// ```
    /// use calendar::DayOfWeek;
    /// assert_eq!(DayOfWeek::from_name(" sunday"), Some(DayOfWeek::Sunday));
    /// assert_eq!(DayOfWeek::from_name("Caturday"), None);
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::ALL
            .into_iter()
            .find(|d| d.as_str().eq_ignore_ascii_case(name))
    }

    /// The name of the day, in lowercase (e.g., `"monday"`)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monday => "monday",
            Self::Tuesday => "tuesday",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
            Self::Saturday => "saturday",
            Self::Sunday => "sunday",
        }
    }

    /// The day of the week `n_days` after this one
    ///
    /// ```
    /// use calendar::DayOfWeek;
    /// assert_eq!(DayOfWeek::Sunday.add_days(1), DayOfWeek::Monday);
    /// assert_eq!(DayOfWeek::Monday.add_days(365), DayOfWeek::Tuesday);
    /// ```
    pub fn add_days(&self, n_days: usize) -> Self {
        let i = Self::ALL.iter().position(|d| d == self).unwrap_or(0);
        Self::ALL[(i + n_days) % 7]
    }
}
