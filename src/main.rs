extern crate chrono;
extern crate clap;
extern crate failure;
extern crate keyring;
extern crate rpassword;

#[macro_use]
extern crate serde_derive;

mod date_helper;
mod feiertage;
mod redmine;

use chrono::Datelike;
use date_helper::*;

fn month_span_from_args(
    v: &clap::ArgMatches,
) -> Result<(chrono::NaiveDate, chrono::NaiveDate), failure::Error> {
    let from = v
        .value_of("from")
        .ok_or(failure::err_msg("from not given as argument"))?
        .parse()?;
    let to = v
        .value_of("to")
        .ok_or(failure::err_msg("to not given as argument"))?
        .parse()?;

    Ok((from, to))
}

fn main() -> Result<(), failure::Error> {
    use clap::{App, Arg};

    let from_txt = format!("{}", current_month());
    let to_txt = format!("{}", next_month(current_month()).pred());

    let matches = App::new("Redmine-Stundentafel")
        .version("0.0")
        .author("Olaf Leidinger<olaf.leidinger@indurad.com>")
        .about("Zeigt die Soll- sowie die Ist-Stundenzahl an")
        .arg(
            Arg::with_name("tf")
                .short("z")
                .long("teilzeit")
                .value_name("FAKTOR")
                .help("Skalierungsfaktor für Wochenstunden")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("username")
                .value_name("USERNAME")
                .help("Username for redmine login")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("from")
                .short("f")
                .long("from")
                .value_name("DATE")
                .help("Startdatum für Zeitabfrage")
                .default_value(&from_txt)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .short("t")
                .long("to")
                .value_name("DATE")
                .help("Enddatum für Zeitabfrage")
                .default_value(&to_txt),
        )
        .get_matches();

    let (from, to) = month_span_from_args(&matches)?;

    let tz_factor = 8.0 * matches.value_of("tf").unwrap().parse::<f32>()?;

    let service = "redmine.indurad.x";
    let username = matches.value_of("user").unwrap();

    let keyring = keyring::Keyring::new(&service, &username);

    let password = keyring.get_password().unwrap_or_else(|_| {
        let pw =
            rpassword::prompt_password_stderr(&format!("Password for {}: ", username)).unwrap();
        keyring
            .set_password(&pw)
            .unwrap_or_else(|e| println!("Couldn't store password to keyring, I'm sorry: {}", e));
        pw
    });

    let feiertage =
        years_in_range(from, to).fold(std::collections::BTreeMap::new(), |mut accum, year| {
            accum.extend(feiertage::get_holidays(year, feiertage::Bundesland::NW).unwrap());
            accum
        });

    let is_no_holiday = |d: &chrono::NaiveDate| !feiertage.contains_key(d);

    {
        println!("|_.Monat\t|_.Arbeitstage\t|_.Sollstunden\t|_.Red.stunden\t|_.Differenz |");
        let mut cur = from;

        while cur < to {
            let last_day_in_month = next_month(cur).pred();
            let hours_sum =
                redmine::HoursSpent::range(cur, last_day_in_month, username, &password)?
                    .fold(0.0, |v, time_res| {
                        v + time_res.ok().and_then(|t| Some(t.hours)).unwrap_or(0.0)
                    });

            let workdays = count_weekdays(cur, next_month(cur), &is_no_holiday);
            let work_hours = workdays as f32 * tz_factor;
            println!(
                "|{}/{:02}     \t|{:>13}\t|{:>13.2}\t|{:>13.2}\t|{:>+11.2} |",
                cur.year(),
                cur.month(),
                workdays,
                work_hours,
                hours_sum,
                hours_sum - work_hours
            );

            cur = next_month(cur);
        }
    }
    Ok(())
}
