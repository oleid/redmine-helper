extern crate chrono;
extern crate failure;
extern crate reqwest;
extern crate serde_json;

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
pub struct Issue {
    id: u64,
}

#[derive(Deserialize, Debug)]
pub struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct Activity {
    id: u64,
    name: String,
}

pub struct HoursSpent {
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
    time_entries: <Vec<TimeEntry> as IntoIterator>::IntoIter,
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
    ) -> Result<Self, failure::Error> {
        Ok(HoursSpent {
            from,
            to,
            time_entries: vec![].into_iter(),
            client: reqwest::Client::new(),
            page: 0,
            per_page: 100,
            total: 0,
            credentials: (user.to_owned(), password.to_owned()),
        })
    }

    fn try_next(&mut self) -> Result<Option<TimeEntry>, failure::Error> {
        if let Some(entry) = self.time_entries.next() {
            return Ok(Some(entry));
        }

        if self.page > 0 && self.page * self.per_page >= self.total {
            return Ok(None);
        }

        self.page += 1;
        let url = format!("https://{}/time_entries.json?user_id=me&set_filter=1&limit={}&period_type=2&from={}&to={}&page={}",
                          env!("REDMINE_SERVER_NAME"), // TODO: This should really be a config setting, but a compile time env is good enough for now
                          self.per_page,
                          self.from,
                          self.to,
                          self.page);
        let mut req = self
            .client
            .get(&url)
            .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
            .send()?;
        if req.status() != 200 {
            panic!("Unexpected http status : {}", req.status())
        }

        let response = req.json::<ApiResponse>()?;
        self.time_entries = response.time_entries.into_iter();
        self.total = response.total_count;
        self.per_page = response.limit; // redmine seems to ignore the arg, if we request "too many".
        Ok(self.time_entries.next())
    }
}

impl Iterator for HoursSpent {
    type Item = Result<TimeEntry, failure::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(dep)) => Some(Ok(dep)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
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
