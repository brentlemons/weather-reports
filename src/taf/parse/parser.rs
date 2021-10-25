use crate::taf::builder::*;
use crate::utils::wind::*;
use crate::utils::visibility::*;


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

        rule visibility() -> &'input str // metars have an option for NDV and NCD; do tafs?
            = visibility: (
                vis_no_info()
                /
                vis_meters()
                /
                vis_statute_miles()
            ) { visibility }

        rule vis_no_info() -> &'input str 
            = vis_no_info: $(slash()*<4>) { vis_no_info } // four slashes -> no visibility info available

        rule vis_meters() -> &'input str 
            = vis_meters: $(numeric()*<4>) { vis_meters } // four numbers -> visibility in meters

        rule vis_statute_miles() -> &'input str 
            = vis_statute_miles: $(
                $(
                    $("P6") // greater than 6 SM {P6SM}
                    /
                    $(numeric() slash() numeric()) // fractional number SM {1/2SM}
                    /
                    $(numeric() " " numeric() slash() numeric()) // whole and fractional number SM {1 1/2SM}
                    /
                    $(numeric()) // whole number SM {3SM}
                ) "SM"
            ) { vis_statute_miles }

        rule wind() -> Wind = wind_direction:wind_direction() wind_speed:wind_speed() wind_gust:wind_gust()? "KT" {
            WindBuilder::new()
                .with_direction(wind_direction)
                .with_speed(wind_speed)
                .with_gust_speed(wind_gust)
                .build()
        }

        rule wind_direction() -> &'input str = wind_direction:$(alpha_numeric()*<3>) {
            wind_direction
        }

        rule wind_speed() -> &'input str = wind_speed:$(numeric()*<2>) {
            wind_speed
        }

        rule wind_gust() -> &'input str = "G" wind_gust:$(numeric()*<2>) {
            wind_gust
        }

        rule numeric() -> &'input str = numeric:$(['0'..='9']) { numeric }
        rule aplha() -> &'input str = alpha:$(['A'..='Z']) { alpha }
        rule alpha_numeric() -> &'input str = alpha_numeric:$(['A'..='Z' | '0'..='9']) { alpha_numeric }
        rule slash() -> &'input str = slash:$(['/']) { slash }

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
