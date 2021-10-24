use crate::taf::builder::*;

peg::parser! {
    pub grammar weather_reports() for str {
        /// [TAF](https://en.wikipedia.org/wiki/Terminal_aerodrome_forecast) parser
        pub rule taf() -> Taf =
                    whitespace()
                    station:icao_identifier() whitespace()
                    issue_time:issue_time() whitespace()
                    valid_times:valid_times() whitespace()
                    conditions:conditions() 
                    last: transition() whitespace()
                    next: conditions()
                    end()
                    {
                        println!("last: *{}*", last);
                        TafBuilder::new(station, issue_time, valid_times)
                            .with_conditions("conditions")
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

        rule transition() -> &'input str = single:$("FM" ['0'..='9']*<6>) {
            println!("> _{}_", single);
            single
        }

//		("^(?<wind>(?<direction>\\w{3}|\\d{3})(?<speed>\\d{2})(?:G(?<gustSpeed>\\d{2}))?KT)?(?:\\s)?(?<visibility>(?:\\d{4})|(?:(?:(?:P6)|(?:[0-6])|(?:[0-6]\\s[0-9]/[0-9])|(?:[0-9]/[0-9]))SM))?(?<other>.*)$");

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
