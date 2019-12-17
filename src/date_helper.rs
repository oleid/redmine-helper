extern crate chrono;

use chrono::{Datelike, NaiveDate};
use std::collections::BTreeSet;
use std::ops::Range;

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
    debug!("count_weekdays: from: {}, to: {} = {}", from, to, count);

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

pub struct Absence {
    planned_absence: BTreeSet<NaiveDate>,
    vacation_days: BTreeSet<NaiveDate>,
}

impl Absence {
    pub fn new(
        planned_absence: impl Iterator<Item = NaiveDate>,
        vacation_days: impl Iterator<Item = NaiveDate>,
    ) -> Absence {
        use std::iter::FromIterator;

        let planned_absence = BTreeSet::from_iter(planned_absence);
        let vacation_days = BTreeSet::from_iter(vacation_days);

        debug!("Absence: planned {:?}", planned_absence);
        debug!("Absence: vacation_days {:?}", vacation_days);

        Absence {
            planned_absence,
            vacation_days,
        }
    }

    pub fn workdays_and_absence(&self, from: NaiveDate, to: NaiveDate) -> (usize, usize) {
        let is_no_holiday = |d: &chrono::NaiveDate| !self.vacation_days.contains(d);

        let is_no_holiday_and_not_absent = |d: &chrono::NaiveDate| {
            !self.vacation_days.contains(d) && !self.planned_absence.contains(d)
        };

        let workdays = count_weekdays(from, to, &is_no_holiday);
        let days_of_absence = workdays - count_weekdays(from, to, &is_no_holiday_and_not_absent);

        (workdays, days_of_absence)
    }
}

#[test]
fn bug_report_planned_absence_too_short() {
    let to_date = |day| NaiveDate::from_ymd(2019, 12, day);

    let planned_absence = (21..32).map(to_date);
    let vacation_days = [25, 26].iter().cloned().map(to_date);

    let absence = Absence::new(planned_absence, vacation_days);
    let (workdays, days_of_absence) =
        absence.workdays_and_absence(to_date(1), next_month(to_date(1)));

    assert_eq!(workdays, 20);
    assert_eq!(days_of_absence, 5);
}
