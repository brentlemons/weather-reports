use crate::taf::builder::*;
use crate::utils::*;


peg::parser! {
    pub grammar weather_reports() for str {
        /// [TAF](https://en.wikipedia.org/wiki/Terminal_aerodrome_forecast) parser
        pub rule taf() -> Taf =
                    whitespace()
                    station:icao_identifier() whitespace()
                    issue_time:issue_time() whitespace()
                    valid_times:valid_times() whitespace()
                    current: condition() whitespace()
                    conditions:conditions() 
                    last: transition() whitespace()
                    next: conditions()
                    end()
                    {
                        println!("current: *{}*", current);
                        println!("last: *{}*", last);
                        TafBuilder::new(station, issue_time, valid_times)
                            .with_conditions(&conditions)
                            .build()
            }

        pub rule icao_identifier() -> &'input str = $(quiet!{letter() letter_or_digit()*<3>} / expected!("ICAO identifier"));
        pub rule issue_time() -> &'input str = $(quiet!{$(['0'..='9']*<6>) "Z"});
        pub rule valid_times() -> &'input str = $(quiet!{$(['0'..='9']*<2>) $(['0'..='9']*<2>) "/" $(['0'..='9']*<2>) $(['0'..='9']*<2>)});

        pub rule conditions() -> String = stuff:single() ++ " " {

            println!("--> .{}.", stuff.len());
            for x in &stuff {
                println!("-> .{}.", x);
            }
            let string = stuff.join(" ");
            println!("=> ,{},", string);
            string
        };
        pub rule end() -> &'input str = $(quiet!{[_]* ![_]});

        rule single() -> &'input str = single:$(&(transition()) / ['A'..='Z' | '0'..='9']+) {
            println!("> _{}_", single);
            single
        }

        rule condition() -> &'input str = wind:wind() " " visibility:visibility() {
            println!("wind: {:#?}", wind);
            println!("visibility: {}", visibility);
            "stuff"
        }

        rule transition() -> &'input str = single:$("FM" ['0'..='9']*<6>) {
            println!("> _{}_", single);
            single
        }

        rule visibility() -> &'input str = visibility:($(['0'..='9']*<4>) / $( (("P6") / (['0'..='6']) / (['0'..='6']" "['0'..='9']"/"['0'..='9']) "SM"))) {
            // ((?:\\d{4})|      (?:(?:(?:P6)  |  (?:[0-6])  |  (?:[0-6]\\s[0-9]/[0-9])  |  (?:[0-9]/[0-9]))       SM)      )
            visibility
        }

        rule wind() -> Wind = wind_direction:wind_direction() wind_speed:wind_speed() wind_gust:wind_gust()? "KT" {
            WindBuilder::new()
                .with_direction(wind_direction)
                .with_speed(wind_speed)
                .with_gust_speed(wind_gust)
                .build()
            // println!("wd: {} | ws: {} | wg: {}", wind_direction, wind_speed, wind_gust.unwrap_or("no gust"));
            // "stuff"
        }

        rule wind_direction() -> &'input str = wind_direction:$(['A'..='Z' | '0'..='9']*<3>) {
            wind_direction
        }

        rule wind_speed() -> &'input str = wind_speed:$(['0'..='9']*<2>) {
            wind_speed
        }

        rule wind_gust() -> &'input str = "G" wind_gust:$(['0'..='9']*<2>) {
            wind_gust
        }

//		("^((?:\\s)?(?<visibility>(?:\\d{4})|(?:(?:(?:P6)|(?:[0-6])|(?:[0-6]\\s[0-9]/[0-9])|(?:[0-9]/[0-9]))SM))?(?<other>.*)$");

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
