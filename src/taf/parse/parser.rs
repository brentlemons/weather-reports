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
                    {
                        TafBuilder::new(station, issue_time, valid_times)
                            .with_conditions(conditions)
                            .build()
            }

        pub rule icao_identifier() -> &'input str = $(quiet!{letter() letter_or_digit()*<3>} / expected!("ICAO identifier"));
        pub rule issue_time() -> &'input str = $(quiet!{$(['0'..='9']*<6>) "Z"});
        pub rule valid_times() -> &'input str = $(quiet!{$(['0'..='9']*<2>) $(['0'..='9']*<2>) "/" $(['0'..='9']*<2>) $(['0'..='9']*<2>)});

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
