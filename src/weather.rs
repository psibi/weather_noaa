use nom::{branch::alt, combinator::map_res};
use nom::multi::{many1, many0};
use nom::bytes::complete::{tag_no_case, take_till};
use nom::character::complete::space1;
use nom::character::complete::{char, newline};
use std::char;
use nom::IResult;
use nom::bytes::complete::tag;
use std::{convert::TryFrom, str::FromStr};
use thiserror::Error;
use nom::error::*;
use reqwest;

#[derive(PartialEq, Debug)]
pub struct WeatherInfo {
    pub station: Option<Station>, //  done
    pub weather_time: WeatherTime, // done
    pub wind: WindInfo,            // done
    pub visibility: String,        // after string
    pub sky_condition: String,     // after string
    pub weather: Option<String>,           // after string
    pub temperature: Temperature, // done
    pub dewpoint: Temperature,    // done
    pub humidity: String,
    pub pressure: i16,          // Pressure in Hectopascal Pressure Unit
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("data store disconnected")]
    ReqwestError(reqwest::Error),
    #[error("Error from Nom: `{0}`")]
    NomError(String)
}

impl From<reqwest::Error> for WeatherError {
    fn from(error: reqwest::Error) -> Self {
        WeatherError::ReqwestError(error)
    }
}

impl From<nom::error::Error<&str>> for WeatherError {
    fn from(error: nom::error::Error<&str>) -> Self {
        WeatherError::NomError(error.to_string())
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for WeatherError {
    fn from(error: nom::Err<nom::error::Error<&str>>) -> Self {
        let str: String = format!("{}", error);
        WeatherError::NomError(str)
    }
}


fn parse_weather_str(i: &str) -> IResult<&str, Option<String>> {
    let (i, k) = many0(tag("Weather: "))(i)?;
    if k.is_empty() {
        return Ok((i, None))
    }
    let (i, weather) = take_till(|c| c == '\n')(i)?;
    let (i,_) = newline(i)?;
    Ok((i, Some(weather.into())))
}


pub async fn get_weather(station_code: String) -> Result<WeatherInfo, WeatherError> {
    let noaa_url = format!("https://tgftp.nws.noaa.gov/data/observations/metar/decoded/{}.TXT", station_code);
    let res =
        reqwest::get(noaa_url).await?;
    let body = res.text().await?;
    let (_, result) = parse_weather(&body)?;
    Ok(result)
}


pub fn parse_weather(i: &str) -> IResult<&str, WeatherInfo> {
    let (i, station) = parse_station(i)?;
    let (i, _) = newline(i)?;
    let (i, weather_time) = parse_time(i)?;
    let (i, _) = newline(i)?;
    let (i, wind) = parse_windinfo(i)?;
    let (i, _) = newline(i)?;
    let (i, _) = tag("Visibility: ")(i)?;
    let (i, visibility) = take_till(|c| c == '\n')(i)?;
    let (i, _) = newline(i)?;
    let (i, _) = tag("Sky conditions: ")(i)?;
    let (i, sky_condition) = take_till(|c| c == '\n')(i)?;
    let (i, _) = newline(i)?;
    let (i, weather) = parse_weather_str(i)?;
    let (i, _) = tag("Temperature:")(i)?;
    let (i, temperature) = parse_temperature(i)?;
    let (i, _) = newline(i)?;
    let (i, _) = tag("Dew Point:")(i)?;
    let (i, dewpoint) = parse_temperature(i)?;
    let (i, _) = newline(i)?;
    let (i, _) = tag("Relative Humidity: ")(i)?;
    let (i, humidity) = take_till(|c| c == '\n')(i)?;
    let (i, _) = newline(i)?;
    let (i, pressure) = parse_pressure(i)?;
    let winfo = WeatherInfo {
        station,
        weather_time,
        wind,
        visibility: visibility.into(),
        sky_condition: sky_condition.into(),
        weather,
        temperature,
        dewpoint,
        humidity: humidity.into(),
        pressure
    };
    Ok((i, winfo))
}



#[derive(PartialEq, Debug)]
pub struct Temperature {
    celsius: i16,
    fahrenheit: i16,
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

fn spaces(input: &str) -> IResult<&str, &str> {
    space1(input)
}

fn parse_pressure(input: &str) -> IResult<&str, i16> {
    let (i, _) = tag("Pressure (altimeter): ")(input)?;
    let (i, _) = take_till(|c| c == '(')(i)?;
    let (i, _) = char('(')(i)?;
    let (i, pressure) = map_res(take_till(char::is_whitespace), |i:&str| i.parse())(i)?;
    let (i, _) = take_till(|c| c == '\n')(i)?;
    Ok((i, pressure))
}

pub fn parse_windinfo(i: &str) -> IResult<&str, WindInfo> {

    fn calm_parser(i: &str) -> IResult<&str, WindInfo> {
        let (i,_) = many1(tag("Wind: Calm:0"))(i)?;
        Ok((i, WindInfo::default()))
    }

    fn wind_from_parser(i: &str) -> IResult<&str, WindInfo> {
        let (i, _) = tag("Wind: from the ")(i)?;
        let (i, cardinal) = take_till(char::is_whitespace)(i)?;
        let (i, _) = spaces(i)?;
        let (i, _) = char('(')(i)?;
        let (i, azimuth) = take_till(char::is_whitespace)(i)?;
        let (i, _) = tag(" degrees) at ")(i)?;
        let (i, mph) = take_till(char::is_whitespace)(i)?;
        let (i, _) = tag(" MPH (")(i)?;
        let (i, knots) = take_till(char::is_whitespace)(i)?;
        let (i, _) = take_till(|c| c == '\n')(i)?;
        let wind_info = WindInfo {
            cardinal: cardinal.into(),
            azimuth: azimuth.into(),
            mph: mph.into(),
            knots: knots.into()
        };
        Ok((i, wind_info))
    }

    fn wind_var_parser(i: &str) -> IResult<&str, WindInfo> {
        let (i, _) = tag("Wind: Variable at ")(i)?;
        let (i, mph) = take_till(char::is_whitespace)(i)?;
        let (i, _) = tag(" MPH (")(i)?;
        let (i, knots) = take_till(char::is_whitespace)(i)?;
        let (i, _) = take_till(|c| c == '\n')(i)?;
        let mut wind_info = WindInfo::default();
        wind_info.mph = mph.into();
        wind_info.knots = knots.into();
        Ok((i, wind_info))
    }

    alt((calm_parser, wind_from_parser, wind_var_parser))(i)
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

fn parse_temperature(i: &str) -> IResult<&str, Temperature> {
    let (i, _) = spaces(i)?;
    let (i, fahrenheit) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
    let (i, _) = tag(" F (")(i)?;
    let (i, celsius) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
    let (i, _) = take_till(|c| c == '\n')(i)?;
    let temperature = Temperature {
        fahrenheit: fahrenheit,
        celsius: celsius
    };
    Ok((i, temperature))
}

#[derive(PartialEq, Debug)]
pub struct WeatherTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub time: String,
}

pub fn parse_time(i: &str) -> IResult<&str, WeatherTime> {
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
    let (i, _) = char(' ')(i)?;
    let (i, time) = take_till(|c| c == '\n')(i)?;
    Ok((
        i,
        WeatherTime {
            year: y,
            month: m,
            day: d,
            time: time.to_owned(),
        }
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
            Ok(("", wtime))
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

        let china_info = WindInfo {
            cardinal: "NNW".into(),
            azimuth: "340".into(),
            mph: "16".into(),
            knots: "14".into()
        };

        assert_eq!(
            parse_windinfo("Wind: from the NNW (340 degrees) at 16 MPH (14 KT):0"),
            Ok(("", china_info))
        )
    }

    #[test]
    fn test_temperature() {
        let temp = Temperature {
            fahrenheit: 78,
            celsius: 26
        };
        assert_eq!(
            parse_temperature(" 78 F (26 C)"),
            Ok(("", temp))
        );

        let temp = Temperature {
            fahrenheit: 66,
            celsius: 19
        };

        assert_eq!(
            parse_temperature(" 66 F (19 C)"),
            Ok(("", temp))
        );
    }

    #[test]
    fn test_pressure() {
        assert_eq!(
            parse_pressure("Pressure (altimeter): 29.62 in. Hg (1003 hPa)"),
            Ok(("", 1003))
        );
    }

    #[test]
    fn test_weather_str() {
        assert_eq!(
            parse_weather_str("Weather: light drizzle; partial fog\n"),
            Ok(("", Some("light drizzle; partial fog".into())))
        );

        // assert_eq!(
        //     parse_weather_str(""),
        //     Ok(("", None))
        // );

        // assert_eq!(
        //     parse_weather_str("non_existent"),
        //     Ok(("", None))
        // );
    }


    #[test]
    fn retrieve_test_weather() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
        let future = rt.block_on(async {get_weather("ZSQD".into()).await});
        assert!(future.is_err());
    }

    #[test]
    fn test_vobl_weather() {
        let weather = "Station name not available
May 16, 2021 - 06:30 AM EDT / 2021.05.16 1030 UTC
Wind: from the SSW (200 degrees) at 12 MPH (10 KT) (direction variable):0
Visibility: 4 mile(s):0
Sky conditions: partly cloudy
Temperature: 80 F (27 C)
Dew Point: 66 F (19 C)
Relative Humidity: 61%
Pressure (altimeter): 29.80 in. Hg (1009 hPa)
extra";
        let winfo =  WeatherInfo {
            station: None,
            weather_time: WeatherTime {
                year: 2021,
                month: 5,
                day: 16,
                time: "1030 UTC".into()
            },
            wind: WindInfo {
                cardinal: "SSW".into(),
                azimuth: "200".into(),
                mph: "12".into(),
                knots: "10".into()
            },
            visibility: "4 mile(s):0".into(),
            sky_condition: "partly cloudy".into(),
            weather: None,
            temperature: Temperature {
                fahrenheit: 80,
                celsius: 27
            },
            dewpoint: Temperature {
                fahrenheit: 66,
                celsius: 19
            },
            humidity: "61%".into(),
            pressure: 1009,
        };

        assert_eq!(parse_weather(weather),
        Ok(("\nextra", winfo)));

    }

    #[test]
    fn test_weather() {
        let weather = "Qingdao, China (ZSQD) 36-04N 120-20E 77M
Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC
Wind: from the NNW (340 degrees) at 16 MPH (14 KT):0
Visibility: 1 mile(s):0
Sky conditions: overcast
Weather: widespread dust
Temperature: 64 F (18 C)
Dew Point: 42 F (6 C)
Relative Humidity: 45%
Pressure (altimeter): 29.65 in. Hg (1004 hPa)";
        let winfo =  WeatherInfo {
            station: Some( Station { place: "Qingdao".into(), country: "China".into()}),
            weather_time: WeatherTime {
                year: 2021,
                month: 3,
                day: 28,
                time: "0800 UTC".into()
            },
            wind: WindInfo {
                cardinal: "NNW".into(),
                azimuth: "340".into(),
                mph: "16".into(),
                knots: "14".into()
            },
            visibility: "1 mile(s):0".into(),
            sky_condition: "overcast".into(),
            weather: Some("widespread dust".into()),
            temperature: Temperature {
                fahrenheit: 64,
                celsius: 18
            },
            dewpoint: Temperature {
                fahrenheit: 42,
                celsius: 6
            },
            humidity: "45%".into(),
            pressure: 1004,
        };

        assert_eq!(parse_weather(weather),
        Ok(("", winfo)));

        let weather2 = "Qingdao, China (ZSQD) 36-04N 120-20E 77M
Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC
Wind: from the NNW (340 degrees) at 16 MPH (14 KT):0
Visibility: 1 mile(s):0
Sky conditions: overcast
Weather: widespread dust
Temperature: 64 F (18 C)
Dew Point: 42 F (6 C)
Relative Humidity: 45%
Pressure (altimeter): 29.65 in. Hg (1004 hPa)
extra";
        let winfo2 =  WeatherInfo {
            station: Some( Station { place: "Qingdao".into(), country: "China".into()}),
            weather_time: WeatherTime {
                year: 2021,
                month: 3,
                day: 28,
                time: "0800 UTC".into()
            },
            wind: WindInfo {
                cardinal: "NNW".into(),
                azimuth: "340".into(),
                mph: "16".into(),
                knots: "14".into()
            },
            visibility: "1 mile(s):0".into(),
            sky_condition: "overcast".into(),
            weather: Some("widespread dust".into()),
            temperature: Temperature {
                fahrenheit: 64,
                celsius: 18
            },
            dewpoint: Temperature {
                fahrenheit: 42,
                celsius: 6
            },
            humidity: "45%".into(),
            pressure: 1004,
        };

        assert_eq!(parse_weather(weather2),
        Ok(("\nextra", winfo2)))
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
