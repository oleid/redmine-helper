extern crate chrono;
extern crate failure;
extern crate reqwest;
extern crate serde_json;

use directories::ProjectDirs;
use std::collections::btree_map::BTreeMap;
use std::fs::File;

#[derive(Deserialize, Debug, Serialize)]
struct ApiResponse {
    #[serde(flatten)]
    inner: BTreeMap<String, HolidayInfo>,
}

#[derive(Deserialize, Debug, Serialize)]
struct HolidayInfo {
    datum: chrono::NaiveDate,
    hinweis: String,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Bundesland {
    BE, // Berlin
    BB, // 	Brandenburg (Potsdam)
    BW, // 	Baden-Württemberg (Stuttgart)
    BY, // 	Bayern (München)
    HB, // 	Bremen
    HH, // 	Hamburg
    HE, // 	Hessen (Wiesbaden)
    MV, // 	Mecklenburg-Vorpommern (Schwerin)
    NI, // 	Niedersachsen (Hannover)
    NW, // 	Nordrhein-Westfalen (Düsseldorf)
    RP, // 	Rheinland-Pfalz (Mainz)
    SH, // 	Schleswig-Holstein (Kiel)
    SL, // 	Saarland (Saarbrücken)
    SN, // 	Sachsen (Dresden)
    ST, // 	Sachsen-Anhalt (Magdeburg)
    TH, //	Thüringen (Erfurt)
}

fn query(year: i32, state: Bundesland) -> Result<ApiResponse, failure::Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://feiertage-api.de/api/?jahr={}&nur_land={:?}",
        year, state
    );

    let mut req = client.get(&url).send()?;

    debug!("Answer from {} : {}", url, req.status());

    if req.status() != 200 {
        return Err(failure::err_msg(format!(
            "Unexpected http status {} when reading {} ",
            req.status(),
            url
        )));
    }

    req.json::<ApiResponse>()
        .map(|response| {
            persist(year, state, &response).unwrap_or_default();
            response
        })
        .map_err(|e| e.into())
}

fn load_cache(year: i32, state: Bundesland) -> Result<ApiResponse, failure::Error> {
    if let Some(proj_dirs) = ProjectDirs::from("org", "Leidingerware", "redmine-helper") {
        let cache_file = proj_dirs
            .cache_dir()
            .join(format!("feiertage_{}_{:?}.json", year, state));

        debug!("Trying cache file: {}", cache_file.display());

        let file = File::open(&cache_file)?;

        serde_json::from_reader::<File, ApiResponse>(file).map_err(|e| e.into())
    } else {
        Err(failure::err_msg("Could not load cache file"))
    }
}

fn persist(year: i32, state: Bundesland, response: &ApiResponse) -> Result<(), failure::Error> {
    if let Some(proj_dirs) = ProjectDirs::from("org", "Leidingerware", "redmine-helper") {
        let cache_dir = proj_dirs.cache_dir();
        std::fs::create_dir_all(cache_dir)?;

        let cache_file = cache_dir.join(format!("feiertage_{}_{:?}.json", year, state));

        debug!("Trying to write cache file: {}", cache_file.display());

        let file = File::create(&cache_file)?;

        serde_json::to_writer(file, response).map_err(|e| e.into())
    } else {
        Err(failure::err_msg("Could not load cache file"))
    }
}

pub fn get_holidays(
    year: i32,
    state: Bundesland,
) -> Result<BTreeMap<chrono::NaiveDate, String>, failure::Error> {
    load_cache(year, state)
        .or_else(|_| query(year, state))
        .and_then(|response| {
            Ok(response
                .inner
                .into_iter()
                .map(|(k, v)| (v.datum, k.into()))
                .collect())
        })
}

#[test]
fn test_feiertage() {
    let data = r#"{
   "Neujahrstag":{
      "datum":"2018-01-01",
      "hinweis":""
   },
   "Karfreitag":{
      "datum":"2018-03-30",
      "hinweis":""
   },
   "Ostermontag":{
      "datum":"2018-04-02",
      "hinweis":""
   },
   "Tag der Arbeit":{
      "datum":"2018-05-01",
      "hinweis":""
   },
   "Christi Himmelfahrt":{
      "datum":"2018-05-10",
      "hinweis":""
   },
   "Pfingstmontag":{
      "datum":"2018-05-21",
      "hinweis":""
   },
   "Fronleichnam":{
      "datum":"2018-05-31",
      "hinweis":""
   },
   "Tag der Deutschen Einheit":{
      "datum":"2018-10-03",
      "hinweis":""
   },
   "Allerheiligen":{
      "datum":"2018-11-01",
      "hinweis":""
   },
   "1. Weihnachtstag":{
      "datum":"2018-12-25",
      "hinweis":""
   },
   "2. Weihnachtstag":{
      "datum":"2018-12-26",
      "hinweis":""
   }
}"#;

    let _r: ApiResponse = serde_json::from_str(data).unwrap();
}
