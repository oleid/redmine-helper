use anyhow::Context;
use std::collections::btree_map::BTreeMap;

type Day = chrono::NaiveDate;

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

pub fn get_days_of_absence(from: Day, to: Day) -> Result<Vec<chrono::NaiveDate>, anyhow::Error> {
    use directories::ProjectDirs;

    // Get config directory in the platform specific default paths
    if let Some(proj_dirs) = ProjectDirs::from("org", "Leidingerware", "redmine-helper") {
        let config_file = proj_dirs.config_dir().join("absence.json");

        use std::fs::File;

        let file = File::open(&config_file).with_context(|| {
            format!(
                "Could not parse days of absence configuration at {config_file:#?}.
     Please create file; you can check check git repo
     https://github.com/oleid/redmine-helper for examples."
            )
        })?;
        let absence: AbsenceConfig = serde_json::from_reader(file).with_context(|| {
            format!("Could not parse days of absence configuration at {config_file:#?}",)
        })?;

        Ok(absence
            .to_days()
            .into_iter()
            .filter(|day| day >= &from && day <= &to)
            .collect())
    } else {
        Ok(Vec::new())
    }
}
