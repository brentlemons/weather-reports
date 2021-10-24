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

#[derive(Debug)]
pub struct Wind {
    is_variable_direction: bool,
    is_gusting: bool,
    direction: Option<u32>,
    speed: Option<u32>,
    gust_speed: Option<u32>,
}

pub struct WindBuilder<'a> {
    variable: bool,
    direction: &'a str,
    speed: &'a str,
    gust_speed: Option<&'a str>,
}

impl<'a> WindBuilder<'a> {

    pub fn new() -> WindBuilder<'a> {
        WindBuilder {
            variable: false,
            direction: Default::default(),
            speed: Default::default(),
            gust_speed: Default::default(),
        }
    }

    pub fn is_variable(&mut self) -> &mut Self {
        self.variable = true;
        self
    }

    pub fn with_direction(&mut self, direction: &'a str) -> &mut Self {
        self.direction = direction;
        self
    }

    pub fn with_speed(&mut self, speed: &'a str) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn with_gust_speed(&mut self, gust_speed: Option<&'a str>) -> &mut Self {
        self.gust_speed = gust_speed;
        self
    }

    pub fn build(&self) -> Wind {
        Wind {
            is_variable_direction: self.variable,
            direction: match self.direction.parse::<u32>() {
                Ok(direction) => Some(direction),
                Err(error) => None
            },
        
            speed: match self.speed.parse::<u32>() {
                Ok(speed) => Some(speed),
                Err(error) => None
            },
        
            gust_speed: match self.gust_speed {
                Some(gust_speed) => Some(self.gust_speed.unwrap().parse::<u32>().unwrap()),
                None => None
            },

            is_gusting: match self.gust_speed {
                Some(gust_speed) => true,
                None => false
            },
        }
    }

}

