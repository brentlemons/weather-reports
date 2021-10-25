use chrono::{DateTime, Utc, Duration, Datelike, TimeZone};
use regex::Regex;

pub fn ddhhmm_to_datetime(ddhhmm: &str) -> DateTime<Utc> {
    let rg = Regex::new(r"^(?P<dd>[0-9]{2})(?P<hh>[0-9]{2})(?P<mm>[0-9]{2})?Z?$").unwrap();

    match rg.captures(ddhhmm) {
        Some(timestamp) => {
            let dd = timestamp.name("dd").unwrap().as_str().parse::<u32>().unwrap();
            let hh = timestamp.name("hh").unwrap().as_str().parse::<u32>().unwrap();
            let mm = match timestamp.name("mm") {
                Some(mm) => mm.as_str().parse::<u32>().unwrap(),
                None => 0
            };
        
            let now = Utc::now();

            let mut m = now.month();
            let mut y = now.year();

            if dd != now.day() {
                let tomorrow = now + Duration::days(1);
                let yesterday = now + Duration::days(-1);
        
                if dd == tomorrow.day() {
                    m = tomorrow.month();
                    y = tomorrow.year();
                } else if dd == yesterday.day() {
                    m = yesterday.month();
                    y = yesterday.year();
                }
            }
        
            if hh >= 24 {
                Utc.ymd(y, m, dd).and_hms(hh-24, mm, 0) + Duration::days(1)
            } else {
                Utc.ymd(y, m, dd).and_hms(hh, mm, 0)
            }        
        },
        None => {Utc::now()}
    }

}
