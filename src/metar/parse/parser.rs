use std::convert::TryFrom;
use uom::{
    si::angle::degree,
    si::f64::{Angle, Length, Pressure, ThermodynamicTemperature, Velocity},
    si::length::{decimeter, foot, kilometer, meter, mile, millimeter},
    si::pressure::{hectopascal, inch_of_mercury},
    si::thermodynamic_temperature::degree_celsius,
    si::velocity::{kilometer_per_hour, knot, meter_per_second},
};

use crate::tokens::*;

peg::parser! {
    pub grammar weather_reports() for str {
        /// [METAR](https://en.wikipedia.org/wiki/METAR) parser
        pub rule metar() -> MetarReport<'input> =
                    whitespace()
                    report_name()? whitespace()
                    pre_observation_flags:observation_flag() ** whitespace() whitespace()
                    identifier:icao_identifier() whitespace()
                    observation_time:observation_time()? whitespace()
                    observation_validity_range:observation_validity_range()? whitespace()
                    // Some stations incorrectly place METAR here
                    report_name()? whitespace()
                    observation_flags:observation_flag() ** whitespace() whitespace()
                    wind:wind()? whitespace()
                    pre_temperatures:temperatures()? whitespace()
                    visibility:visibility()? whitespace()
                    runway_visibilities:runway_visibility() ** whitespace() whitespace()
                    pre_recent_weather:recent_weather_sequence()? whitespace()
                    weather:weather_sequence()? whitespace()
                    cloud_cover:cloud_cover() ** whitespace() whitespace()
                    cavok:("CAVOK" whitespace())?
                    temperatures:temperatures()? whitespace()
                    pressure:pressure()? whitespace()
                    // Some stations also report the altimeter setting in a different unit and/or Q Field Elevation, discard it
                    pressure() ** whitespace() whitespace()
                    weather_post_pressure:weather_sequence()? whitespace()
                    cloud_cover_post_pressure:cloud_cover() ** whitespace() whitespace()
                    temperatures_post_pressure:temperatures()? whitespace()
                    accumulated_rainfall:accumulated_rainfall()? whitespace()
                    recent_weather:recent_weather_sequence()? whitespace()
                    cloud_cover_post_recent_weather:cloud_cover() ** whitespace() whitespace()
                    temperatures_post_recent_weather:temperatures()? whitespace()
                    // Military stations often report these
                    color:color()? whitespace()
                    // Some stations report runway visibility after pressure
                    runway_visibilities_post_pressure:runway_visibility() ** whitespace() whitespace()
                    runway_reports:runway_report() ** whitespace() whitespace()
                    water_conditions:water_conditions()? whitespace()
                    trends:trend()** whitespace() whitespace()
                    remark:$((":RMK" / "R MK"/ "RMK" / "REMARK") [^'$']*)?
                    maintenance_needed:quiet!{"$"}? whitespace()
                    // Consumes trailing garbage characters
                    quiet!{"/"*} whitespace()
                    // Some machines use = to indicate end of message
                    quiet!{"=" [_]*}? whitespace()
                    {
                MetarReport {
                    identifier,
                    observation_time,
                    observation_validity_range,
                    observation_flags: pre_observation_flags.iter().copied().chain(observation_flags).collect(),
                    wind: wind.flatten(),
                    visibility: visibility.flatten(),
                    runway_visibilities: runway_visibilities.iter().copied().chain(runway_visibilities_post_pressure).flatten().collect(),
                    runway_reports: runway_reports.iter().copied().flatten().collect(),
                    weather: weather.unwrap_or_default().drain(..).chain(weather_post_pressure.unwrap_or_default()).collect(),
                    cloud_cover: cloud_cover.iter().copied().chain(cloud_cover_post_pressure).chain(cloud_cover_post_recent_weather).flatten().collect(),
                    cavok: cavok.is_some(),
                    temperatures: pre_temperatures.flatten().or_else(|| temperatures.flatten()).or_else(|| temperatures_post_pressure.flatten()).or_else(|| temperatures_post_recent_weather.flatten()),
                    pressure: pressure.flatten(),
                    accumulated_rainfall,
                    recent_weather: pre_recent_weather.unwrap_or_default().iter().cloned().chain(recent_weather.unwrap_or_default()).collect(),
                    color,
                    water_conditions,
                    trends,
                    remark,
                    maintenance_needed: maintenance_needed.is_some(),
                }
            }
        rule report_name() -> &'input str = quiet!{$("METAR" / "SPECI")} / expected!("report name");

        pub rule icao_identifier() -> &'input str = $(quiet!{letter() letter_or_digit()*<3>} / expected!("ICAO identifier"));

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

        pub rule observation_time() -> ZuluDateTime = day_of_month:$(digit() digit()) time:zulu_time() is_zulu:"Z"? {
            // TODO: some stations don't include the Z. Not sure if that could mean it is local time and not GMT.
            ZuluDateTime {
                day_of_month: day_of_month.parse().unwrap(),
                time,
                is_zulu: is_zulu.is_some(),
            }
        }
        rule zulu_time() -> ZuluTime = hour:$(digit()*<2>) minute:$(digit()*<2>) {
            ZuluTime {
                hour: hour.parse().unwrap(),
                minute: minute.parse().unwrap(),
            }
        }

        rule observation_validity_range() -> ZuluTimeRange = begin:zulu_time() "/" end:zulu_time() {
            ZuluTimeRange {
                begin,
                end,
            }
        }

        rule observation_flag() -> ObservationFlag = val:$(quiet!{"AUTO" / "NIL" / "COR" / "CCA" / "CCB" / "CCC" / "RTD"} / expected!("observation flag")) { ObservationFlag::try_from(val).unwrap() };

        pub rule wind() -> Option<Wind> =
            direction:$("VRB" / (digit()*<3>))? speed:$(("P" digit()*<2>) / (digit()+ ("." digit()+)?))? peak_gust:$("G" ("//" / digit()+))? unit:windspeed_unit() whitespace() variance:wind_variance()? {
                let speed = speed.map(|speed| speed.trim_start_matches('P').parse().unwrap());
                Some(Wind {
                    direction: direction.filter(|dir| *dir != "VRB").map(|direction| Angle::new::<degree>(direction.parse().unwrap())),
                    speed: speed.map(|speed| match unit {
                        "MPS" => Velocity::new::<meter_per_second>(speed),
                        "KT" | "KTS" | "KTM" => Velocity::new::<knot>(speed),
                        "KMH" => Velocity::new::<kilometer_per_hour>(speed),
                        _ => unreachable!()
                    }),
                    peak_gust: peak_gust.filter(|gusts| *gusts != "G//").map(|gusts| gusts.trim_start_matches('G').parse().unwrap()).map(|gusts| match unit {
                        "MPS" => Velocity::new::<meter_per_second>(gusts),
                        "KT" | "KTS" | "KTM" => Velocity::new::<knot>(gusts),
                        "KMH" => Velocity::new::<kilometer_per_hour>(gusts),
                        _ => unreachable!()
                    }),
                    variance,
                })
            }
            / ("//////" / "/////") windspeed_unit() {
                None
            }
        rule windspeed_unit() -> &'input str = $(quiet!{"MPS" / "KTM" / "KTS" / "KT" / "KMH"} / expected!("velocity unit"))


        rule wind_variance() -> (Angle, Angle) = variance_begin:$(digit()+) "V" variance_end:$(digit()+) {
            (
                Angle::new::<degree>(variance_begin.parse().unwrap()),
                Angle::new::<degree>(variance_end.parse().unwrap()),
            )
        }

        pub rule visibility() -> Option<Visibility> =
            // Some systems will attach a number in front of NDV
            (digit()*) "NDV" visibility_unit()? { None }
            / "////" visibility_unit() { None }
            / "////" "NDV" visibility_unit()? { None }
            / prevailing:raw_visibility() whitespace() minimum_directional:raw_directional_visibility() whitespace() maximum_directional:raw_directional_visibility() {
                Some(Visibility {
                    prevailing: Some(prevailing),
                    minimum_directional: Some(minimum_directional),
                    maximum_directional: Some(maximum_directional),
                })
            }
            / prevailing:raw_visibility() whitespace() minimum_directional:raw_directional_visibility() {
                Some(Visibility {
                    prevailing: Some(prevailing),
                    minimum_directional: Some(minimum_directional),
                    maximum_directional: None,
                })
            }
            / minimum_directional:raw_directional_visibility() whitespace() maximum_directional:raw_directional_visibility() {
                Some(Visibility {
                    prevailing: None,
                    minimum_directional: Some(minimum_directional),
                    maximum_directional: Some(maximum_directional),
                })
            }
            / minimum_directional:raw_directional_visibility() {
                Some(Visibility {
                    prevailing: None,
                    minimum_directional: Some(minimum_directional),
                    maximum_directional: None,
                })
            }
            / prevailing:raw_visibility() {
                Some(Visibility {
                    prevailing: Some(prevailing),
                    minimum_directional: None,
                    maximum_directional: None,
                })
            }
        rule raw_directional_visibility() -> DirectionalVisibility = distance:raw_visibility() direction:compass_direction() {
            DirectionalVisibility {
                direction,
                distance,
            }
        }
        rule raw_visibility() -> RawVisibility =
            out_of_range:out_of_range()? whole:$(digit()+) whitespace() numerator:$(digit()+) "/" denominator:$(digit()+) unit:visibility_unit()? {
                let value = whole.parse::<f64>().unwrap() + numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap();

                let distance = match unit {
                    Some("KM") => Length::new::<kilometer>(value),
                    Some("SM") => Length::new::<mile>(value),
                    Some("M") | None => Length::new::<meter>(value),
                    _ => unreachable!()
                };
                RawVisibility {
                    distance,
                    out_of_range,
                }
            }
            / out_of_range:out_of_range()? numerator:$(digit()+) "/" denominator:$(digit()+) unit:visibility_unit()? {
                let value = numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap();
                let distance = match unit {
                    Some("KM") => Length::new::<kilometer>(value),
                    Some("SM") => Length::new::<mile>(value),
                    Some("M") | None => Length::new::<meter>(value),
                    _ => unreachable!()
                };
                RawVisibility {
                    distance,
                    out_of_range,
                }
            }
            / out_of_range:out_of_range()? value:$(digit()+) unit:visibility_unit()? {
                let value = value.parse::<f64>().unwrap();
                let distance = match unit {
                    Some("KM") => Length::new::<kilometer>(value),
                    Some("SM") => Length::new::<mile>(value),
                    Some("M") | None => Length::new::<meter>(value),
                    _ => unreachable!()
                };
                RawVisibility {
                    distance,
                    out_of_range,
                }
            }

        rule compass_direction() -> CompassDirection = val:$(quiet!{"NE" / "NW" / "N" / "SE" / "SW" / "S" / "E" / "W"} / expected!("8-point compass direction")) {
            CompassDirection::try_from(val).unwrap()
        }
        rule visibility_unit() -> &'input str = whitespace() val:$(quiet!{"M" / "KM" / "SM"} / expected!("visibility unit")) &required_whitespace_or_eof() { val }

        pub rule runway_visibility() -> Option<RunwayVisibility<'input>> =
            "R" designator:designator() "/" !runway_report_info() range:raw_runway_visibility_range() trend:visibility_trend()? "/"? {
                Some(RunwayVisibility {
                    designator,
                    visibility: VisibilityType::Varying {
                        lower: range.0,
                        upper: range.1,
                    },
                    trend,
                })
            }
            / "R" designator:designator() "/" !runway_report_info() visibility:raw_runway_visibility() trend:visibility_trend()? "/"? {
                Some(RunwayVisibility {
                    designator,
                    visibility: VisibilityType::Fixed(visibility),
                    trend,
                })
            }
            // A varying number of slashes and a missing designator has been observed here
            / "R" designator:designator()? ("/////" "/"*) &required_whitespace_or_eof() {
                None
            }
        rule raw_runway_visibility_range() -> (RawVisibility, RawVisibility) = lower_out_of_range:out_of_range()? lower_value:$(digit()+) "V" upper_out_of_range:out_of_range()? upper_value:$(digit()+) unit:$("FT")? {
            let lower_value = lower_value.parse::<f64>().unwrap();
            let upper_value = upper_value.parse::<f64>().unwrap();
            if let Some("FT") = unit {
                (
                    RawVisibility {
                        distance: Length::new::<foot>(lower_value),
                        out_of_range: lower_out_of_range,
                    },
                    RawVisibility {
                        distance: Length::new::<foot>(upper_value),
                        out_of_range: upper_out_of_range,
                    },
                )
            } else {
                (
                    RawVisibility {
                        distance: Length::new::<meter>(lower_value),
                        out_of_range: lower_out_of_range,
                    },
                    RawVisibility {
                        distance: Length::new::<meter>(upper_value),
                        out_of_range: upper_out_of_range,
                    },
                )
            }
        }
        rule raw_runway_visibility() -> RawVisibility = out_of_range:out_of_range()? value:$(digit()+) unit:$("FT")? {
            let value = value.parse::<f64>().unwrap();
            if let Some("FT") = unit {
                RawVisibility {
                    distance: Length::new::<foot>(value),
                    out_of_range,
                }
            } else {
                RawVisibility {
                    distance: Length::new::<meter>(value),
                    out_of_range,
                }
            }
        }
        rule out_of_range() -> OutOfRange = val:$(quiet!{"M" / "P"} / expected!("bound")) { OutOfRange::try_from(val).unwrap() };
        rule visibility_trend() -> VisibilityTrend = "/"? val:$(quiet!{("D" / "N" / "U")} / expected!("visibility trend")) { VisibilityTrend::try_from(val.trim_start_matches('/')).unwrap() };

        pub rule runway_report() -> Option<RunwayReport<'input>> =
            "R" designator:designator() "/" report_info:runway_report_info() {
                Some(RunwayReport {
                    designator,
                    report_info,
                })
            }
        rule runway_report_info() -> RunwayReportInfo =
            "CLRD" friction:$("//" / digit()+) {
                RunwayReportInfo::Cleared {
                    friction: if friction == "//" { None } else { Some(friction.parse::<f64>().unwrap()) }
                }
            }

        rule designator() -> &'input str = $(quiet!{digit()+ ("L"/"C"/"R"/"D")?} / expected!("runway designator"));


        rule recent_weather_sequence() -> Vec<Weather> = recent_weather:recent_weather() ++ whitespace() &required_whitespace_or_eof() {
            recent_weather.iter().cloned().flatten().collect()
        }
        rule recent_weather() -> Option<Weather> =
            "RE" weather:weather() &required_whitespace_or_eof() { Some(weather) }
            / "RE//" &required_whitespace_or_eof() { None }

        rule weather_sequence() -> Vec<Weather> = weather:weather() ++ whitespace() &required_whitespace_or_eof() { weather }

        pub rule weather() -> Weather =
            intensity:intensity() vicinity:"VC"? descriptor:descriptor()? precipitation:precipitation()+ {
                Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor,
                    condition: Some(Condition::Precipitation(precipitation)),
                }
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor()? obscuration:obscuration() {
                Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor,
                    condition: Some(Condition::Obscuration(obscuration)),
                }
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor()? other:other() {
                Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor,
                    condition: Some(Condition::Other(other)),
                }
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor() {
                Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor: Some(descriptor),
                    condition: None,
                }
            }
        rule intensity() -> Intensity = val:$(quiet!{[ '+' | '-' ]} / expected!("intensity"))? { val.map(Intensity::try_from).transpose().unwrap().unwrap_or(Intensity::Moderate) }
        rule descriptor() -> Descriptor =
            val:$(quiet!{
                "MI"
                / "PR"
                / "BC"
                / "DR"
                / "BL"
                / "SH"
                / "TS"
                / "FZ"
            } / expected!("descriptor")) {
                Descriptor::try_from(val).unwrap()
        }

        rule precipitation() -> Precipitation =
            val:$(quiet!{
                "RA"
                / "DZ"
                / "SN"
                / "SG"
                / "IC"
                / "PL"
                / "GR"
                / "GS"
                / "UP"
            } / expected!("precipitation")) {
                Precipitation::try_from(val).unwrap()
        }

        rule obscuration() -> Obscuration =
        val:$(quiet!{
                "FG"
                / "BR"
                / "HZ"
                / "VA"
                / "DU"
                / "FU"
                / "SA"
                / "PY"
            } / expected!("obscuration")) {
                Obscuration::try_from(val).unwrap()
        }

        rule other() -> Other =
            val:$(quiet!{
                "SQ"
                / "PO"
                / "DS"
                / "SS"
                / "FC"
            } / expected!("other weather condition")) {
                Other::try_from(val).unwrap()
        }


        pub rule cloud_cover() -> Option<CloudCover> =
            "/"+ cloud_type:cloud_type() {
                None
            }
            / coverage:cloud_coverage() whitespace() "///" whitespace() cloud_type:cloud_type()? {
                Some(CloudCover {
                    coverage,
                    base: None,
                    cloud_type: cloud_type.flatten(),
                })
            }
            / coverage:cloud_coverage() whitespace() base:$(digit()*<3, 4>) whitespace() "//" required_whitespace_or_eof() {
                Some(CloudCover {
                    coverage,
                    base: Some(Length::new::<foot>(base.parse().unwrap()) * 100.),
                    cloud_type: None,
                })
            }
            / coverage:cloud_coverage() whitespace() base:$(digit()*<3, 4>) whitespace() cloud_type:cloud_type()? {
                Some(CloudCover {
                    coverage,
                    base: Some(Length::new::<foot>(base.parse().unwrap()) * 100.),
                    cloud_type: cloud_type.flatten(),
                })
            }
            / coverage:cloud_coverage() {
                Some(CloudCover {
                    coverage,
                    base: None,
                    cloud_type: None,
                })
            }

        rule cloud_coverage() -> CloudCoverage =
            val:$(quiet!{
                "SKC"
                / "CLR"
                / "NCD"
                / "NSC"
                / "FEW"
                / "FW"
                / "SCT"
                / "SC"
                / "BKN"
                / "OVC"
                / "VV"
            } / expected!("cloud coverage")) {
                CloudCoverage::try_from(val).unwrap()
            }

        rule cloud_type() -> Option<CloudType> =
            val:$(quiet!{"CB" / "TCU" / "CU" / "CI" / "AC" / "ST"} / expected!("cloud type")) { Some(CloudType::try_from(val).unwrap()) }
            / "///" {
                None
            }


        rule temperature() -> ThermodynamicTemperature = minus:(quiet!{"M" / "-"} / expected!("minus"))? temp:$(digit()+) {
            ThermodynamicTemperature::new::<degree_celsius>(if minus.is_some() { -temp.parse::<f64>().unwrap() } else { temp.parse().unwrap() })
        }

        pub rule temperatures() -> Option<Temperatures> =
            air:temperature() ("/" / ".") ("XX" / "//") !(visibility_unit() / windspeed_unit()) {
                Some(Temperatures {
                    air,
                    dewpoint: None,
                })
            }
            / air:temperature() ("/" / ".") dewpoint:temperature()? !(visibility_unit() / windspeed_unit()) {
                Some(Temperatures {
                    air,
                    dewpoint
                })
            }
            / "XX/XX" {
                None
            }

        pub rule pressure() -> Option<Pressure> =
            pressure_unit:pressure_unit() whitespace() pressure:$(digit()+ ("." digit()+)?) {
                match pressure_unit {
                    "A" => Some(Pressure::new::<hectopascal>(pressure.parse().unwrap())),
                    _ => Some(Pressure::new::<inch_of_mercury>(pressure.parse::<f64>().unwrap() / 100.))
                }
            }
            / pressure_unit() whitespace() ("////" / "NIL") { None }
        rule pressure_unit() -> &'input str = $(quiet!{"QFE" / "QNH" / "Q" / "A"} / expected!("pressure unit"));

        rule accumulated_rainfall() -> AccumulatedRainfall = "RF" recent:$(digit()+ "." digit()+) "/" past:$(digit()+ "." digit()+) {
            AccumulatedRainfall {
                recent: Length::new::<millimeter>(recent.parse().unwrap()),
                past: Length::new::<millimeter>(past.parse().unwrap()),
            }
        }

        pub rule color() -> Color =
            is_black:"BLACK"? whitespace() current_color:color_state() whitespace() next_color:color_state() &required_whitespace_or_eof() {
                Color {
                    is_black: is_black.is_some(),
                    current_color,
                    next_color: Some(next_color),
                }
            }
            / is_black:"BLACK"? whitespace() current_color:color_state() &required_whitespace_or_eof() {
                Color {
                    is_black: is_black.is_some(),
                    current_color,
                    next_color: None,
                }
            }
        rule color_state() -> ColorState = val:$(quiet!{"BLU+" / "BLU" / "WHT" / "GRN" / "YLO1" / "YLO2" / "YLO" / "AMB" / "RED"} / expected!("color state")) { ColorState::try_from(val).unwrap() }

        pub rule water_conditions() -> WaterConditions =
            "W" temperature:$("//" / digit()+) "/" "S" surface_state:$("/" / digit()) {
                WaterConditions {
                    temperature: if temperature == "//" { None } else { Some(ThermodynamicTemperature::new::<degree_celsius>(temperature.parse().unwrap()))},
                    surface_state: if surface_state == "/" { None } else { Some(WaterSurfaceState::try_from(surface_state).unwrap()) },
                    significant_wave_height: None,
                }
            }
            / "W" temperature:$("//" / digit()+) "/" "H" wave_height:$("/"+ / digit()+) {
                WaterConditions {
                    temperature: if temperature == "//" { None } else { Some(ThermodynamicTemperature::new::<degree_celsius>(temperature.parse().unwrap()))},
                    surface_state: None,
                    significant_wave_height: if wave_height.starts_with('/') { None } else { Some(Length::new::<decimeter>(wave_height.parse().unwrap())) },
                }
            }

        rule trend() -> Trend =
            $(quiet!{"NOSIG" / "NOISIG" / "N0SIG" / "NOS16" / "NOSING" / "NOSG" / "NSG"} / expected!("trend")) {
                Trend::NoSignificantChange
            }
            /   val:$(quiet!{"BECMG" / "TEMPO"} / expected!("trend")) whitespace()
                time:trend_time()? whitespace()
                wind:wind()? whitespace()
                visibility:visibility()? whitespace()
                weather:weather_sequence()? whitespace()
                "NSW"? whitespace()
                cloud_cover:cloud_cover() ** whitespace() whitespace()
                color_state:color_state()? whitespace() {
                    let trend = TrendReport {
                        time,
                        wind: wind.flatten(),
                        visibility: visibility.flatten(),
                        weather: weather.unwrap_or_default(),
                        cloud_cover: cloud_cover.iter().copied().flatten().collect(),
                        color_state,
                    };
                    match val {
                        "BECMG" => Trend::Becoming(trend),
                        "TEMPO" => Trend::Temporarily(trend),
                        _ => unreachable!()
                    }
            }
        rule trend_time() -> TrendTime = time_type:trend_time_type() time:zulu_time() {
            TrendTime {
                time_type,
                time,
            }
        };
        rule trend_time_type() -> TrendTimeType = val:$(quiet!{"AT" / "FM" / "TL"} / expected!("trend time type")) { TrendTimeType::try_from(val).unwrap() }
    }
}
