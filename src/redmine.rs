use anyhow::{anyhow, Context};

type TimeStamp = chrono::DateTime<chrono::Utc>;

#[derive(Deserialize)]
#[allow(dead_code)]
struct ApiResponse {
    time_entries: Vec<TimeEntry>,
    total_count: i64,
    offset: i64,
    limit: i64,
}

#[derive(Deserialize, Debug)]
pub struct TimeEntry {
    pub id: u64,
    pub project: Project,

    #[serde(default)]
    pub issue: Option<Issue>,
    pub user: User,
    pub activity: Activity,
    pub hours: f32,
    pub comments: String,
    pub spent_on: chrono::NaiveDate,
    pub entity_id: u64,
    pub entity_type: String,
    pub created_on: TimeStamp,
    pub updated_on: TimeStamp,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Issue {
    id: u64,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Project {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Activity {
    id: u64,
    name: String,
}

pub struct HoursSpent {
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
    time_entries: Vec<TimeEntry>,
    client: reqwest::Client,
    page: i64,
    per_page: i64,
    total: i64,
    credentials: (String, String), // TODO: polish credential stuff
}

impl HoursSpent {
    pub fn range(
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
        user: &str,
        password: &str,
        client: reqwest::Client,
    ) -> Self {
        HoursSpent {
            from,
            to,
            time_entries: Vec::new(),
            client,
            page: 0,
            per_page: 100,
            total: 0,
            credentials: (user.to_owned(), password.to_owned()),
        }
    }

    pub(crate) async fn run(mut self) -> Result<Vec<TimeEntry>, anyhow::Error> {
        loop {
            if self.page > 0 && self.page * self.per_page >= self.total {
                break;
            }

            self.page += 1;
            let url = format!("https://{}/time_entries.json?user_id=me&set_filter=1&limit={}&period_type=2&from={}&to={}&page={}",
env!("REDMINE_SERVER_NAME"), // TODO: This should really be a conf
                              self.per_page,
                              self.from,
                              self.to.pred(), // end date not included
                              self.page);
            let req = self
                .client
                .get(&url)
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .await
                .with_context(|| "While attempting to download hours from redmine.")?;

            if req.status() != 200 {
                return Err(anyhow!("Unexpected http status : {}", req.status()));
            }

            let response = req.json::<ApiResponse>().await?;
            self.time_entries.extend(response.time_entries.into_iter());
            self.total = response.total_count;
            self.per_page = response.limit; // redmine seems to ignore the arg, if we request "too many".
        }
        Ok(self.time_entries)
    }
}

#[test]
fn test_deserialize() {
    let data = r#"{
   "time_entries":[
      {
         "id":231460,
         "project":{
            "id":646,
            "name":"34101_iBelt"
         },
         "issue":{
            "id":33956
         },
         "user":{
            "id":136,
            "name":"Olaf Leidinger"
         },
         "activity":{
            "id":18,
            "name":"9. Error analysis\u0026Debugging"
         },
         "hours":4.0,
         "comments":"Fehlersuche iDVR / Volumenfluss",
         "spent_on":"2018-09-05",
         "entity_id":33956,
         "entity_type":"Issue",
         "created_on":"2018-09-05T14:27:56Z",
         "updated_on":"2018-09-05T14:27:56Z"
      },
      {
         "id":231508,
         "project":{
            "id":731,
            "name":"SolutionFramework"
         },
         "user":{
            "id":136,
            "name":"Olaf Leidinger"
         },
         "activity":{
            "id":27,
            "name":"10. Other"
         },
         "hours":0.5,
         "comments":"review",
         "spent_on":"2018-09-05",
         "entity_id":37665,
         "entity_type":"Project",
         "created_on":"2018-09-05T15:23:58Z",
         "updated_on":"2018-09-05T15:23:58Z"
      }
   ],
   "total_count":40,
   "offset":3,
   "limit":2
}"#;

    let _r: ApiResponse = serde_json::from_str(data).unwrap();
}
