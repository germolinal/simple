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

use crate::schedule_trait::Schedule;
use calendar::Date;

pub struct ScheduleConstant<T>(T);

impl<T> Schedule<T> for ScheduleConstant<T>
where
    T: Copy + Sync,
{
    fn get(&self, _date: Date) -> Option<T> {
        Some(self.0)
    }
}

impl<T> ScheduleConstant<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

/* *********** */
/*    TESTS    */
/* *********** */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() -> Result<(), String> {
        // with usize
        let date = Date {
            month: 1,
            day: 1,
            hour: 13.,
        };

        let v: usize = 1;
        let constant = ScheduleConstant(v);
        assert_eq!(constant.get(date).ok_or("no value 0")?, v);
        assert_eq!(constant.get(date).ok_or("no value 1")?, v);
        assert_eq!(constant.get(date).ok_or("no value 2")?, v);

        // With char
        let date = Date {
            month: 1,
            day: 1,
            hour: 13.,
        };

        let v: char = 'a';
        let constant = ScheduleConstant(v);
        assert_eq!(constant.get(date).ok_or("no value 3")?, v);
        assert_eq!(constant.get(date).ok_or("no value 4")?, v);
        assert_eq!(constant.get(date).ok_or("no value 5")?, v);

        // With float
        let date = Date {
            month: 1,
            day: 1,
            hour: 13.,
        };

        let v = 123.1;
        let constant = ScheduleConstant(v);
        assert_eq!(constant.get(date).ok_or("no value 6")?, v);
        assert_eq!(constant.get(date).ok_or("no value 7")?, v);
        assert_eq!(constant.get(date).ok_or("no value 8")?, v);

        Ok(())
    }
}
