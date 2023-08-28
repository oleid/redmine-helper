use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::ops::{Add, Range};

pub fn count_weekdays(
    from: NaiveDate,
    to: NaiveDate,
    condition: &dyn Fn(&NaiveDate) -> bool,
) -> usize {
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

pub fn next_week_monday(date: NaiveDate) -> NaiveDate {
    date.add(Duration::days(match date.weekday() {
        Weekday::Mon => 7,
        Weekday::Tue => 6,
        Weekday::Wed => 5,
        Weekday::Thu => 4,
        Weekday::Fri => 3,
        Weekday::Sat => 2,
        Weekday::Sun => 1,
    }))
}

pub fn current_month() -> NaiveDate {
    let t = chrono::Local::today();

    NaiveDate::from_ymd(t.year(), t.month(), 1)
}

pub fn today() -> NaiveDate {
    chrono::Local::today().naive_local()
}

pub fn years_in_range(from: NaiveDate, to: NaiveDate) -> Range<i32> {
    Range {
        start: from.year(),
        end: to.year() + 1,
    }
}

#[test]
fn test_add_week() {
    let a = NaiveDate::from_ymd(2022, 2, 18);
    let b = NaiveDate::from_ymd(2022, 2, 21);
    assert_eq!(next_week_monday(a), b);
}
