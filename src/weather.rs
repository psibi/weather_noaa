use nom::branch::alt;
use nom::bytes::streaming::{tag_no_case, take, take_till};
use nom::Err::Error;
use nom::IResult;
use std::convert::TryFrom;

#[derive(PartialEq, Debug)]
pub struct WindInfo {
    pub cardinal: String,
    pub azimuth: String,
    pub mph: String,
    pub knots: String,
}

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
    pub year: String,
    pub month: String,
    pub day: String,
    // pub year: u16,
    // pub month: u8,
    // pub day: u8,
    pub time: String,
}

pub fn parse_time(i: &str) -> IResult<&str, Option<WeatherTime>> {
    // Parsers a sample string like this
    // Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC
    let (i, _) = take_till(|c| c == '/')(i)?;
    let (i, _) = take(1usize)(i)?;
    match i.trim().split('.').collect::<Vec<&str>>()[..] {
        [ref y, ref m, ref d] => {
            let mut time = d.split(' ').collect::<Vec<&str>>();
            let day = time.remove(0);
            return Ok((
                i,
                Some(WeatherTime {
                    year: y.to_string(),
                    month: m.to_string(),
                    day: day.to_string(),
                    time: time.join(" "),
                }),
            ));
        }
        _ => return Ok((i, None)),
    }
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
        assert_eq!(
            parse_time("Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC"),
            Ok(("", None))
        );
    }
}
