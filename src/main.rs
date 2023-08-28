#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate prettytable;

mod absence;
mod date_helper;
mod feiertage;
mod program_config;
mod redmine;

use crate::date_helper::*;
use crate::program_config::Settings;
use anyhow::Context;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use prettytable::{format, Cell, Row, Table};
use reqwest::Client;
use std::{collections::BTreeSet, iter::FromIterator};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let s = program_config::get_settings()?;

    let planned_absence =
        BTreeSet::from_iter(absence::get_days_of_absence(s.from, s.to)?.into_iter());
    let vacation_days = get_vacation_days_for(&s).await?;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![
        "Monat",
        "Arbeitstage",
        "davon Abwesend",
        "Sollstunden",
        "Redmine-Stunden",
        "Differenz"
    ]);

    let table_data =
        query_redmine(&s, planned_absence, vacation_days, reqwest::Client::new()).await?;

    for (start_date, end_date, data) in table_data.iter() {
        let last_day_included = end_date.pred();

        if last_day_included - *start_date == Duration::zero() {
            continue;
        }

        table.add_row(make_row(
            Cell::new(&if (*end_date - *start_date).num_days() > 7 {
                format!("{}/{:02}", start_date.year(), start_date.month())
            } else if start_date.weekday() == Weekday::Mon {
                format!(
                    "{} KW{:02}",
                    start_date.year(),
                    start_date.iso_week().week()
                )
            } else if start_date.month() == last_day_included.month() {
                format!(
                    "{}/{:02}/{:02} - {:02}",
                    start_date.year(),
                    start_date.month(),
                    start_date.day(),
                    last_day_included.day()
                )
            } else {
                format!(
                    "{}/{:02}/{:02} - {:02}/{:02}",
                    start_date.year(),
                    start_date.month(),
                    start_date.day(),
                    last_day_included.month(),
                    last_day_included.day()
                )
            })
            .style_spec("i"),
            &data,
        ));
    }

    if table_data.len() > 1 {
        let sum = table_data
            .iter()
            .fold(RowData::default(), |accum, (_, _, data)| {
                accum + data.clone()
            });

        table.add_empty_row();
        table.add_row(make_row(Cell::new("Gesamt").style_spec("b"), &sum));
    }

    table.printstd();

    Ok(())
}

async fn query_redmine(
    s: &Settings,
    planned_absence: BTreeSet<NaiveDate>,
    vacation_days: BTreeSet<NaiveDate>,
    http_client: Client,
) -> anyhow::Result<Vec<(NaiveDate, NaiveDate, RowData)>> {
    let mut tasks: Vec<tokio::task::JoinHandle<_>> = Vec::new();
    for (start_date, end_date) in get_date_ranges_to_query(&s) {
        let vacation_days = vacation_days.clone();
        let planned_absence = planned_absence.clone();
        let client = http_client.clone();
        let settings = s.clone();
        tasks.push(tokio::spawn(async move {
            compute_table_row(
                vacation_days,
                planned_absence,
                settings,
                client,
                start_date,
                end_date,
            )
            .await
        }));
    }

    let mut table_data = Vec::new();
    for task in tasks {
        let res = task.await??;
        table_data.push(res);
    }
    Ok(table_data)
}

/// vec of ranges, 2nd item won't be included in query
fn get_date_ranges_to_query(s: &Settings) -> Vec<(NaiveDate, NaiveDate)> {
    let mut cur = s.from;
    let mut dates = Vec::new();

    loop {
        if cur < s.to {
            cur = if next_month(cur) < s.to {
                dates.push((cur, next_month(cur)));
                next_month(cur)
            } else if next_week_monday(cur) < s.to {
                dates.push((cur, next_week_monday(cur)));
                next_week_monday(cur)
            } else {
                dates.push((cur, s.to.succ()));
                s.to.succ()
            };
        } else {
            break;
        }
    }
    dates
}

async fn compute_table_row(
    vacation_days: BTreeSet<NaiveDate>,
    planned_absence: BTreeSet<NaiveDate>,
    s: Settings,
    client: Client,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> anyhow::Result<(NaiveDate, NaiveDate, RowData)> {
    let is_no_holiday = |d: &chrono::NaiveDate| !vacation_days.contains(d);

    let is_no_holiday_and_not_absent =
        |d: &chrono::NaiveDate| !vacation_days.contains(d) && !planned_absence.contains(d);

    let workdays = count_weekdays(start_date, end_date, &is_no_holiday);
    let days_of_absence =
        workdays - count_weekdays(start_date, end_date, &is_no_holiday_and_not_absent);

    let work_hours = (workdays - days_of_absence) as f32 * s.tz_factor;
    Ok((
        start_date,
        end_date,
        RowData {
            workdays,
            days_of_absence,
            work_hours,
            ..RowData::default()
        } + row_data_from_redmine(start_date, end_date, &s, client.clone()).await?,
    ))
}

async fn get_vacation_days_for(s: &Settings) -> anyhow::Result<BTreeSet<NaiveDate>> {
    let mut accum: BTreeSet<NaiveDate> = std::collections::BTreeSet::new();
    for year in years_in_range(s.from, s.to) {
        accum.extend(
            feiertage::get_holidays(year, feiertage::Bundesland::NW)
                .await
                .with_context(|| format!("When querying holidays for {year}"))?
                .keys(),
        );
    }
    Ok(accum)
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

async fn row_data_from_redmine(
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    settings: &program_config::Settings,
    client: reqwest::Client,
) -> anyhow::Result<RowData> {
    let redmine_hours = redmine::HoursSpent::range(
        start_date,
        end_date,
        &settings.username,
        &settings.password,
        client,
    )
    .run()
    .await?
    .into_iter()
    .fold(0.0, |v, time_res| v + time_res.hours);

    Ok(RowData {
        redmine_hours,
        ..RowData::default()
    })
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
