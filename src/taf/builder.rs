use crate::tokens::*;
use crate::utils::times::*;
use chrono::{DateTime, Utc};

#[derive(Debug)]
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
    valid_times: &'a str,
    conditions: Option<&'a str>,
}

impl<'a> TafBuilder<'a> {

    pub fn new(station: &'a str, issue_time: &'a str, valid_times: &'a str) -> TafBuilder<'a> {
        TafBuilder {
            station: station,
            issue_time: issue_time,
            valid_times: valid_times,
            conditions: None,
        }
    }

    pub fn with_conditions(&mut self, conditions: &'a str) -> &mut Self {
        self.conditions = Some(conditions);
        self
    }

    pub fn build(&self) -> Taf {
        Taf {
            station: String::from(self.station),
            issue_time: ddhhmm_to_datetime(self.issue_time),
            valid_times: {
                ValidDateTimes {
                    start: ddhhmm_to_datetime(&self.valid_times[0..4]),
                    end: ddhhmm_to_datetime(&self.valid_times[5..])
                }
            },
            conditions: match self.conditions {
                Some(conditions) => Some(String::from(conditions)),
                None => None
            },
        }
    }

}

