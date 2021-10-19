use chrono::{DateTime, Utc, Duration, Datelike, TimeZone};

pub fn ddhhmm_to_datetime(dd: u32, hh: u32, mm: u32) -> DateTime<Utc> {
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

}