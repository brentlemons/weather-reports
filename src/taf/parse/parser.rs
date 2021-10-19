use crate::tokens::*;
use crate::utils::*;
use chrono::{DateTime, Utc};
use crate::taf::builder::*;

peg::parser! {
    pub grammar weather_reports() for str {
        /// [TAF](https://en.wikipedia.org/wiki/Terminal_aerodrome_forecast) parser
        pub rule taf() -> TafReport<'input> =
                    whitespace()
                    station:icao_identifier() whitespace()
                    issue_time:issue_time() whitespace()
                    valid_times:valid_times() whitespace()
                    conditions:conditions()
                    {
                        let taf = TafBuilder::new();
                TafReport {
                    station,
                    issue_time,
                    valid_times,
                    conditions
                }
            }

        pub rule icao_identifier() -> &'input str = $(quiet!{letter() letter_or_digit()*<3>} / expected!("ICAO identifier"));

        pub rule issue_time() -> DateTime<Utc>
            = day:$(['0'..='9']*<2>) hour:$(['0'..='9']*<2>) minute:$(['0'..='9']*<2>) "Z" {
                ddhhmm_to_datetime(
                    day.parse::<u32>().unwrap(), 
                    hour.parse::<u32>().unwrap(), 
                    minute.parse::<u32>().unwrap()
                )
            }

        pub rule valid_times() -> ValidDateTimes
            = start_dd:$(['0'..='9']*<2>) start_hh:$(['0'..='9']*<2>) "/" end_dd:$(['0'..='9']*<2>) end_hh:$(['0'..='9']*<2>) {
                ValidDateTimes {
                    start: ddhhmm_to_datetime(
                        start_dd.parse::<u32>().unwrap(),
                        start_hh.parse::<u32>().unwrap(),
                        0
                    ),
                    end: ddhhmm_to_datetime(
                        end_dd.parse::<u32>().unwrap(),
                        end_hh.parse::<u32>().unwrap(),
                        0
                    )
                }
            }

        pub rule conditions() -> &'input str = $(quiet!{[_]* ![_]})

        /// This must also consume garbage characters from irregular reports
        pub rule whitespace() = required_whitespace()?
        rule required_whitespace_or_eof() = (required_whitespace() / ![_])
        rule required_whitespace() =
            quiet!{
                (
                    (" " ("/"+ " ")+)
                    / (" " ("M" " ")+)
                    / " "
                    / "\r\n"
                    / "\n"
                    / "\t"
                    ">"
                )+
            }
            / expected!("whitespace");
        rule digit() -> &'input str = quiet!{$(['0'..='9'])} / expected!("digit");
        rule letter() -> &'input str = quiet!{$(['A'..='Z'])} / expected!("letter");
        rule letter_or_digit() -> &'input str = letter() / digit();

    }
}
