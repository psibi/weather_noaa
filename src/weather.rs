use nom::branch::alt;
use nom::bytes::streaming::{tag_no_case, take_till};
use nom::lib::std::ops::Fn;
use nom::{
    alt,
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple,
    IResult,
};
use std::convert::TryFrom;

pub struct WindInfo {
    cardinal: String,
    azimuth: String,
    mph: String,
    knots: String,
    kmh: String,
    ms: String,
}

pub struct WeatherInfo {
    station: Option<Station>,
    year: String,
    month: String,
    day: String,
    hour: String,
    windinfo: WindInfo,
    visibility: String,
    skyCondition: String,
    weather: String,
    temp_celsius: String,
    temp_fahrenheit: String,
    dewpoint_celsius: String,
    dewpoint_fahrenheit: String,
    humidity: i16,
    pressure: i16,
}

#[derive(PartialEq, Debug)]
pub struct Station {
    place: String,
    country: String,
}

impl TryFrom<&str> for Station {
    type Error = String;

    fn try_from(i: &str) -> Result<Self, Self::Error> {
        match i.split(',').collect::<Vec<&str>>()[..] {
            [ref s1, ref s2] => {
                let mut country = s2.to_string();
                match country.split('(').collect::<Vec<&str>>()[..] {
                    [ref c, ..] => {
                        country = c.trim().to_string();
                    }
                    _ => {}
                }
                Ok(Station {
                    place: s1.to_string(),
                    country,
                })
            }
            _ => Err(format!("Failuer parsing {}", i)),
        }
    }
}

pub fn parse_missing_station_name(i: &str) -> IResult<&str, Option<Station>> {
    let result = alt((
        tag_no_case("Station name not available"),
        take_till(|c| c == '\n'),
    ))(i);
    match result {
        Ok((input, output)) => {
            let station: Result<Station, String> = Station::try_from(output);
            match station {
                Ok(stat) => Ok((input, Some(stat))),
                Err(_) => Ok((input, None)),
            }
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        Err as NomErr,
    };

    #[test]
    fn test_scheme() {
        assert_eq!(
            parse_missing_station_name("Station name not available"),
            Ok(("", None))
        );
        let station = Station {
            place: "Qingdao".to_string(),
            country: "China".to_string(),
        };
        assert_eq!(
            parse_missing_station_name("Qingdao, China (ZSQD) 36-04N 120-20E 77M\n"),
            Ok(("dd", Some(station)))
        );
    }
}
