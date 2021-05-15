use nom::{branch::alt, combinator::map_res};
use nom::multi::many1;
use nom::bytes::streaming::{tag_no_case, take_till};
use nom::character::streaming::char;
use nom::IResult;
use nom::bytes::complete::tag;
use std::{convert::TryFrom, str::FromStr};
use thiserror::Error;
use nom::error::*;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("Parsing failed for the item `{0}`")]
    ParseError(String),
}

// todo: implement error handling


#[derive(PartialEq, Debug)]
pub struct WeatherInfo {
    pub station: Option<Station>,
    pub year: String,
    pub month: String,
    pub day: String,
    pub hour: String,
    pub wind: WindInfo,
    pub visibility: String,
    pub sky_condition: String,
    pub weather: String,
    pub temp_celsius: String,
    pub temp_fahrenheit: String,
    pub dewpoint_celsius: String,
    pub dewpoint_fahrenheit: String,
    pub humidity: i16,
    pub pressure: i16,
}

#[derive(PartialEq, Debug)]
pub struct Station {
    place: String,
    country: String,
}

impl FromStr for Station {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
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

#[derive(PartialEq, Debug, Clone)]
pub struct WindInfo {
    pub cardinal: String,
    pub azimuth: String,
    pub mph: String,
    pub knots: String,
}

impl Default for WindInfo {
    fn default() -> Self {
        WindInfo {
            cardinal: "μ".into(),
            azimuth: "μ".into(),
            mph: "0".into(),
            knots: "0".into()
        }
    }
}

pub fn parse_windinfo(i: &str) -> IResult<&str, WindInfo> {
    let (i,_) = many1(tag("Wind: Calm:0"))(i)?;
    Ok((i, WindInfo::default()))
}

pub fn parse_station(i: &str) -> IResult<&str, Option<Station>> {
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

#[derive(PartialEq, Debug)]
pub struct WeatherTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub time: String,
}

pub fn parse_time(i: &str) -> IResult<&str, Option<WeatherTime>> {
    // Parsers a sample string like this
    // Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC
    let (i, _) = take_till(|c| c == '/')(i)?;
    let (i, _) = char('/')(i)?;
    let (i, _) = char(' ')(i)?;
    let (i, y) = map_res(
        take_till(|c| c == '.'),
        |s: &str| s.parse::<u16>(),
    )(i)?;
    let (i, _) = char('.')(i)?;
    let (i, m) = map_res(
        take_till(|c| c == '.'),
        |s: &str| s.parse::<u8>(),
    )(i)?;
    let (i, _) = context(
        "Trying to parse day",
        char('.'),
    )(i)?;
    let (i, d) = map_res(
        take_till(|c| c == ' '),
        |s: &str| s.parse::<u8>(),
    )(i)?;
    let (time, _) = char(' ')(i)?;
    Ok((
        "",
        Some(WeatherTime {
            year: y,
            month: m,
            day: d,
            time: time.to_owned(),
        })
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_station() {
        assert_eq!(parse_station("Station name not available"), Ok(("", None)));
        let station = Station {
            place: "Qingdao".to_string(),
            country: "China".to_string(),
        };
        assert_eq!(
            parse_station("Qingdao, China (ZSQD) 36-04N 120-20E 77M\n"),
            Ok(("\n", Some(station)))
        );
    }

    #[test]
    fn test_time() {
        let wtime = WeatherTime {
            year: 2021,
            month: 3,
            day: 28,
            time: "0800 UTC".into(),
        };
        assert_eq!(
            parse_time("Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC"),
            Ok(("", Some(wtime)))
        );
    }

    #[test]
    fn test_wind_info() {
        let winfo = WindInfo {
            cardinal: "μ".into(),
            azimuth: "μ".into(),
            mph: "0".into(),
            knots: "0".into()
        };
        assert_eq!(
            parse_windinfo("Wind: Calm:0"),
            Ok(("", winfo.clone()))
        );
        assert!(
            parse_windinfo("Wind: unexpected").is_err()
        );
    }
}

// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/VOBL.TXT
// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/VOBL.xml
// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/VOBL.json

// With station names
// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/ZSSS.TXT
// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/ZSQD.TXT
// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/ZSPD.TXT
// https://tgftp.nws.noaa.gov/data/observations/metar/decoded/YMML.TXT (aus)

// Qingdao, China (ZSQD) 36-04N 120-20E 77M
// Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC
// Wind: from the NNW (340 degrees) at 16 MPH (14 KT):0
// Visibility: 1 mile(s):0
// Sky conditions: overcast
// Weather: widespread dust
// Temperature: 64 F (18 C)
// Dew Point: 42 F (6 C)
// Relative Humidity: 45%
// Pressure (altimeter): 29.65 in. Hg (1004 hPa)
// ob: ZSQD 280800Z 34007MPS 2000 DU OVC020 18/06 Q1004 BECMG TL0930 3000
// cycle: 8

// Reimplementatin of https://github.com/jaor/xmobar/blob/master/src/Xmobar/Plugins/Monitors/Weather.hs
