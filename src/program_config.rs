#[derive(Deserialize, Debug, Default)]
struct Config {
    pub username: Option<String>,
    pub teilzeitfaktor: Option<f32>,
}

fn read_config() -> Config {
    use directories::ProjectDirs;

    // Get config directory in the platform specific default paths
    if let Some(proj_dirs) = ProjectDirs::from("org", "Leidingerware", "redmine-helper") {
        let config_file = proj_dirs.config_dir().join("config.json");

        use std::fs::File;

        File::open(&config_file)
            .map(|file| {
                serde_json::from_reader::<File, Config>(file).expect(&format!(
                    "Could not parse config file at {:#?}, fix or delete ;)",
                    config_file
                ))
            })
            .unwrap_or(Config::default())
    } else {
        Config::default()
    }
}

fn get_settings_and_cmdline_parser() -> (clap::ArgMatches<'static>, Config) {
    use clap::{App, Arg};

    let config = read_config();

    (
        App::new("Redmine-Stundentafel")
            .version("0.1.1")
            .author("Olaf Leidinger<oleid@meschaet.de>")
            .about("Zeigt die Soll- sowie die Ist-Stundenzahl an")
            .arg(
                Arg::with_name("tf")
                    .short("z")
                    .long("teilzeit")
                    .value_name("FAKTOR")
                    .help("Skalierungsfaktor für Wochenstunden")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("user")
                    .short("u")
                    .long("username")
                    .value_name("USERNAME")
                    .help("Username for redmine login")
                    .required(config.username.is_none())
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("from")
                    .short("f")
                    .long("from")
                    .value_name("DATE")
                    .help("Startdatum für Zeitabfrage. Standard = Monatsanfang")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("to")
                    .short("t")
                    .long("to")
                    .value_name("DATE")
                    .help("Enddatum für Zeitabfrage (einschließlich), Standard = Ende der Woche"),
            )
            .get_matches(),
        config,
    )
}

#[derive(Clone)]
pub struct Settings {
    pub from: chrono::NaiveDate,
    pub to: chrono::NaiveDate,
    pub tz_factor: f32,
    pub username: String,
    pub password: String,
}

pub fn get_settings() -> Result<Settings, anyhow::Error> {
    let (matches, config) = get_settings_and_cmdline_parser();

    let (from, to) = month_span_from_args(&matches)?;

    let tz_factor = 8.0
        * matches
            .value_of("tf")
            .map(|v| v.parse::<f32>().unwrap())
            .or(config.teilzeitfaktor)
            .unwrap_or(1.0);

    let service = env!("REDMINE_SERVER_NAME"); // TODO: This should really be a conf
    let username = matches
        .value_of("user")
        .map(|v| v.to_owned())
        .or(config.username)
        .unwrap();

    let password = {
        let keyring = keyring::Entry::new(&service, &username);

        keyring.get_password().unwrap_or_else(|_| {
            let pw = rpassword::prompt_password_stderr(&format!("Password for {}: ", &username))
                .unwrap();
            keyring.set_password(&pw).unwrap_or_else(|e| {
                println!("Couldn't store password to keyring, I'm sorry: {}", e)
            });
            pw
        })
    };

    Ok(Settings {
        from,
        to,
        tz_factor,
        username,
        password,
    })
}

fn month_span_from_args(
    v: &clap::ArgMatches,
) -> Result<(chrono::NaiveDate, chrono::NaiveDate), anyhow::Error> {
    use crate::date_helper::*;

    let alt_from = current_month();
    let alt_to = next_week_monday(today()).pred();

    let from = v
        .value_of("from")
        .map(|v| v.parse())
        .unwrap_or(Ok(alt_from))?;
    let to = v.value_of("to").map(|v| v.parse()).unwrap_or(Ok(alt_to))?;

    Ok((from, to))
}
