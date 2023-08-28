use anyhow::{anyhow, Context};
use std::collections::btree_map::BTreeMap;

#[derive(Deserialize, Debug)]
struct ApiResponse {
    #[serde(flatten)]
    inner: BTreeMap<String, HolidayInfo>,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct HolidayInfo {
    datum: chrono::NaiveDate,
    hinweis: String,
}

#[derive(Debug)]
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

pub async fn get_holidays(
    year: i32,
    state: Bundesland,
) -> Result<BTreeMap<chrono::NaiveDate, String>, anyhow::Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://feiertage-api.de/api/?jahr={}&nur_land={:?}",
        year, state
    );

    let req = client
        .get(&url)
        .send()
        .await
        .with_context(|| "While downloading feiertage")?;

    if req.status() != 200 {
        return Err(anyhow!("Unexpected http status : {}", req.status()));
    }

    let response = req.json::<ApiResponse>().await?;

    Ok(response
        .inner
        .into_iter()
        .map(|(k, v)| (v.datum, k.into()))
        .collect())
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
