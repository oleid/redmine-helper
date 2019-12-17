extern crate chrono;
extern crate directories;
extern crate failure;
extern crate serde_json;

use chrono::NaiveDate as Day;
use std::collections::btree_map::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
struct AbsenceConfig {
    #[serde(flatten)]
    inner: BTreeMap<String, Absence>,
}

impl AbsenceConfig {
    fn to_days(&self) -> Vec<Day> {
        self.inner.values().fold(Vec::new(), |mut accum, v| {
            accum.extend(v.clone().into_iter());
            accum
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)] // hides the variant type in serialization format
enum Absence {
    SingleDay(Day),
    MultiDay { first_day: Day, last_day: Day },
}

impl IntoIterator for Absence {
    type Item = Day;
    type IntoIter = AbsenceIterator;

    fn into_iter(self) -> Self::IntoIter {
        let index = match self {
            Absence::SingleDay(day) => day,
            Absence::MultiDay {
                first_day,
                last_day: _,
            } => first_day,
        };
        AbsenceIterator {
            absence: self,
            index,
        }
    }
}

struct AbsenceIterator {
    absence: Absence,
    index: Day,
}

impl Iterator for AbsenceIterator {
    type Item = Day;
    fn next(&mut self) -> Option<Day> {
        let limit = match self.absence {
            Absence::SingleDay(day) => day,
            Absence::MultiDay {
                first_day: _,
                last_day,
            } => last_day,
        };

        let result = if self.index <= limit {
            Some(self.index)
        } else {
            None
        };

        self.index = self.index.succ();
        result
    }
}

pub fn get_days_of_absence(from: Day, to: Day) -> Result<Vec<Day>, failure::Error> {
    use directories::ProjectDirs;

    // Get config directory in the platform specific default paths
    if let Some(proj_dirs) = ProjectDirs::from("org", "Leidingerware", "redmine-helper") {
        let config_file = proj_dirs.config_dir().join("absence.json");

        use std::fs::File;

        match File::open(&config_file) {
            Ok(file) => {
                let absence: AbsenceConfig = serde_json::from_reader(file).expect(&format!(
                    "Could not parse days of absence configuration at {:#?}",
                    config_file
                ));
                Ok(extract_days_in_range_inclusive(absence, from, to).collect())
            }
            Err(e) => Err(failure::err_msg(format!(
                "Cannot open the days of absence configuration at {:#?} for reading : {}. \
                 Please make sure it exists and fill it with data. \
                 Check repo for examples.",
                config_file, e
            ))),
        }
    } else {
        Ok(Vec::new())
    }
}

fn extract_days_in_range_inclusive(
    absence: AbsenceConfig,
    from: Day,
    to: Day,
) -> impl Iterator<Item = Day> {
    absence
        .to_days()
        .into_iter()
        .filter(move |day| *day >= from && *day <= to)
}

#[test]
fn bug_report_planned_absence_too_short_1() {
    let to_date = |day| Day::from_ymd(2019, 12, day);

    let whole_month: Vec<Day> = Absence::MultiDay {
        first_day: Day::from_ymd(2019, 12, 1),
        last_day: Day::from_ymd(2020, 1, 1),
    }
    .into_iter()
    .collect();

    let mut expected_result: Vec<Day> = (1..32).map(to_date).collect();
    expected_result.push(Day::from_ymd(2020, 1, 1));

    assert_eq!(whole_month, expected_result);
}

#[test]
fn bug_report_planned_absence_too_short_2() {
    use std::iter::FromIterator;

    let to_date = |day| Day::from_ymd(2019, 12, day);

    let absence = AbsenceConfig {
        inner: BTreeMap::from_iter(
            [(
                "SomeVacation".to_owned(),
                Absence::MultiDay {
                    first_day: Day::from_ymd(2019, 12, 1),
                    last_day: Day::from_ymd(2020, 1, 1),
                },
            )]
            .iter()
            .cloned(),
        ),
    };
    let result = extract_days_in_range_inclusive(
        absence,
        Day::from_ymd(2019, 12, 1),
        Day::from_ymd(2019, 12, 31),
    )
    .collect::<Vec<_>>();

    let expected_result: Vec<Day> = (1..32).map(to_date).collect();

    assert_eq!(result, expected_result);
}
