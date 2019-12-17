extern crate chrono;
extern crate clap;
extern crate directories;
extern crate failure;
extern crate keyring;
extern crate rayon;
extern crate rpassword;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate prettytable;

mod absence;
mod date_helper;
mod feiertage;
mod program_config;
mod redmine;

use chrono::Datelike;
use date_helper::*;
use prettytable::{format, Cell, Row, Table};
use rayon::prelude::*;

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let s = program_config::get_settings()?;
    let planned_absence = absence::get_days_of_absence(s.from, s.to)?.into_iter();
    let vacation_days = years_in_range(s.from, s.to)
        .map(|year| {
            feiertage::get_holidays(year, feiertage::Bundesland::NW)
                .unwrap_or_default()
                .into_iter()
                .map(|(k, _)| k)
        })
        .flatten();
    let absence = Absence::new(planned_absence, vacation_days);

    // rayon is used to perform redmine calls in parallel.
    // Since these tasks are IO bound, it's not really the right tool, but is does the job fine.
    // The number of threads can be a lot higher than the number of cores.
    // But it shouldn't overload the server either.
    rayon::ThreadPoolBuilder::new()
        .num_threads(10)
        .build_global()
        .unwrap();

    let mut table = Table::new();
    {
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row![
            "Monat",
            "Arbeitstage",
            "davon Abwesend",
            "Sollstunden",
            "Redmine-Stunden",
            "Differenz"
        ]);

        let table_data = {
            let mut cur = s.from;
            let mut dates = Vec::new();
            while cur < s.to {
                dates.push(cur);
                cur = next_month(cur);
            }
            dates
        }
        .par_iter()
        .map(|date| {
            let (workdays, days_of_absence) =
                absence.workdays_and_absence(*date, next_month(*date));

            let work_hours = (workdays - days_of_absence) as f32 * s.tz_factor;

            (
                *date,
                RowData {
                    workdays,
                    days_of_absence,
                    work_hours,
                    ..RowData::default()
                } + query_redmine_month(*date, &s),
            )
        })
        .collect::<Vec<(chrono::NaiveDate, RowData)>>();

        for (date, data) in table_data.iter() {
            table.add_row(make_row(
                Cell::new(&format!("{}/{:02}", date.year(), date.month())).style_spec("i"),
                data,
            ));
        }

        if table_data.len() > 1 {
            let sum = table_data
                .iter()
                .fold(RowData::default(), |accum, (_, data)| accum + data.clone());

            table.add_empty_row();
            table.add_row(make_row(Cell::new("Gesamt").style_spec("b"), &sum));
        }
    }
    table.printstd();

    Ok(())
}

#[derive(Debug, Default, Clone)]
struct RowData {
    workdays: usize,
    days_of_absence: usize,
    work_hours: f32,
    redmine_hours: f32,
}

impl std::ops::Add for RowData {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        RowData {
            workdays: self.workdays + other.workdays,
            days_of_absence: self.days_of_absence + other.days_of_absence,
            work_hours: self.work_hours + other.work_hours,
            redmine_hours: self.redmine_hours + other.redmine_hours,
        }
    }
}

fn query_redmine_month(date: chrono::NaiveDate, settings: &program_config::Settings) -> RowData {
    let last_day_in_month = next_month(date).pred();

    debug!(
        "query_redmine_month: from {} to {}",
        date, last_day_in_month
    );

    RowData {
        redmine_hours: redmine::HoursSpent::range(
            date,
            last_day_in_month,
            &settings.username,
            &settings.password,
        )
        .unwrap()
        .fold(0.0, |v, time_res| {
            v + time_res.ok().and_then(|t| Some(t.hours)).unwrap_or(0.0)
        }),
        ..RowData::default()
    }
}

fn make_row(caption: Cell, data: &RowData) -> Row {
    Row::new(vec![
        caption,
        fmt_cell(data.workdays),
        fmt_cell(data.days_of_absence),
        fmt_cell(data.work_hours),
        fmt_cell(data.redmine_hours),
        fmt_cell(data.redmine_hours - data.work_hours),
    ])
}

fn fmt_cell<T>(val: T) -> prettytable::Cell
where
    T: std::fmt::Display + PartialOrd + Default,
{
    let txt = format!("{:.2}", val);

    if val < T::default() {
        Cell::new(&txt).style_spec("rFr")
    } else {
        Cell::new(&txt).style_spec("r")
    }
}
