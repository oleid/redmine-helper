extern crate chrono;

use chrono::{Datelike, NaiveDate};
use std::ops::Range;

pub fn count_weekdays(from: NaiveDate, to: NaiveDate, condition: &Fn(&NaiveDate) -> bool) -> usize {
    use chrono::Datelike;

    let mut cur = from.clone();
    let mut count = 0;

    while cur != to {
        if cur.weekday() != chrono::Weekday::Sat
            && cur.weekday() != chrono::Weekday::Sun
            && condition(&cur)
        {
            count += 1;
        }
        cur = cur.succ();
    }

    count
}

pub fn next_month(date: NaiveDate) -> NaiveDate {
    let year = date.year() + (date.month() / 12) as i32;
    let month = (date.month() % 12) + 1;

    NaiveDate::from_ymd(year, month, 1)
}

pub fn current_month() -> NaiveDate {
    let t = chrono::Local::today();

    NaiveDate::from_ymd(t.year(), t.month(), 1)
}

pub fn years_in_range(from: NaiveDate, to: NaiveDate) -> Range<i32> {
    Range {
        start: from.year(),
        end: to.year() + 1,
    }
}
