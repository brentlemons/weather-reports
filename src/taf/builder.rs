use chrono::{DateTime, Utc};

#[derive(Default, Debug)]
pub struct Taf {
    /// Station [ICAO identifier](https://en.wikipedia.org/wiki/ICAO_airport_code)
    station: String,
    issue_time: DateTime<Utc>,
    valid_times: ValidDateTimes,
    conditions: Option<String>,
}

pub struct TafBuilder<'a> {
    station: &'a str,
    issue_time: &'a str,
    valid_times: DateTime<Utc>,
    conditions: &'a str,
}

impl<'a> TafBuilder<'a> {

    pub fn new(station: String, issue_time: DateTime<Utc>, valid_times: ValidDateTimes) -> TafBuilder<'a> {
        TafBuilder {
            station: station,
            issue_time: issue_time,
            valid_times: valid_times,
            conditions: Default::default(),
        }
    }

    pub fn with_conditions(&mut self, conditions: &'a str) -> &mut Self {
        self.conditions = conditions;
        self
    }

    pub fn build(&self) -> Taf {
        Taf {
            station: self.station,
            issue_time: self.issue_time,
            valid_times: self.valid_times,
            conditions: self.conditions,
        }
    }

}

