use nom::bytes::complete::tag;
use nom::bytes::complete::{tag_no_case, take_till};
use nom::character::complete::space1;
use nom::character::complete::{char, newline};
use nom::combinator::opt;
use nom::error::*;
use nom::multi::{many0, many1};
use nom::IResult;
use nom::{branch::alt, combinator::map_res};
use reqwest::Client;
use std::char;
use std::{convert::TryFrom, str::FromStr};
use thiserror::Error;

/// Weather information for a particular station.
#[derive(PartialEq, Debug)]
pub struct WeatherInfo {
    /// Weather station code. More information about it is present in the [Station metadata page](https://www.ncdc.noaa.gov/data-access/land-based-station-data/station-metadata).
    pub station: Option<Station>,
    /// Timestamp of the weather
    pub weather_time: WeatherTime,
    /// Wind Information
    pub wind: WindInfo,
    /// Visibility Details. Eg: 1 mile(s):0
    pub visibility: String,
    /// Sky condition. Eg: overcast, partly cloudy etc.
    pub sky_condition: Option<String>,
    /// Weather information. Eg: widespread dust, mist
    pub weather: Option<String>,
    /// Temperature
    pub temperature: Temperature,
    /// Dewpoint Temperature. More details [here](https://en.wikipedia.org/wiki/Dew_point)
    pub dewpoint: Temperature,
    /// Relative Humidity. More details [here](https://en.wikipedia.org/wiki/Humidity#Relative_humidity)
    pub relative_humidity: f64,
    /// Pressure in Hectopascal Pressure Unit
    pub pressure: i16,
}

/// The timestamp of the weather data.
#[derive(PartialEq, Debug)]
pub struct WeatherTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub time: String,
}

/// Enum representing the various errors that the library can return.
#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Error from request: `{0}`")]
    ReqwestError(reqwest::Error),
    #[error("Error from Nom: `{0}`")]
    NomError(nom::Err<nom::error::Error<String>>),
}

/// Temperature in both celsius and Fahrenheit units.
#[derive(PartialEq, Debug)]
pub struct Temperature {
    /// Temperature in celsius
    pub celsius: f64,
    /// Temperature in Fahrenheit
    pub fahrenheit: f64,
}

/// Weather station information
#[derive(PartialEq, Debug)]
pub struct Station {
    /// Station place
    pub place: String,
    /// Country where the station is located
    pub country: String,
}

/// Wind Information
#[derive(PartialEq, Debug, Clone)]
pub struct WindInfo {
    /// Cardinal direction. More details [here](https://en.wikipedia.org/wiki/Cardinal_direction)
    pub cardinal: String,
    /// Azimuth. More details [here](https://en.wikipedia.org/wiki/Azimuth#Navigation)
    pub azimuth: f64,
    /// Wind speed in Miles per hour
    pub mph: f64,
    /// Speed in knots. More details [here](https://en.wikipedia.org/wiki/Knot_(unit))
    pub knots: f64,
}

impl From<reqwest::Error> for WeatherError {
    fn from(error: reqwest::Error) -> Self {
        WeatherError::ReqwestError(error)
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for WeatherError {
    fn from(error: nom::Err<nom::error::Error<&str>>) -> Self {
        WeatherError::NomError(error.map(|e| nom::error::Error::new(e.input.to_string(), e.code)))
    }
}

fn parse_weather_str(i: &str) -> IResult<&str, Option<String>> {
    let (i, k) = many0(tag("Weather: "))(i)?;
    if k.is_empty() {
        return Ok((i, None));
    }
    let (i, weather) = take_till(|c| c == '\n')(i)?;
    let (i, _) = newline(i)?;
    Ok((i, Some(weather.into())))
}

pub struct NoaaApp {
    client: Client,
    blocking_client: reqwest::blocking::Client,
}

impl NoaaApp {
    pub fn new() -> Self {
        NoaaApp {
            client: Client::new(),
            blocking_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn with_client(client: Client) -> Self {
        NoaaApp {
            client,
            blocking_client: reqwest::blocking::Client::new(),
        }
    }

    /// This function retrieves the weather information from from the NOAA
    /// observations.
    pub async fn get_weather(&self, station_code: &str) -> Result<WeatherInfo, WeatherError> {
        let noaa_url = format!(
            "https://tgftp.nws.noaa.gov/data/observations/metar/decoded/{}.TXT",
            station_code
        );
        let res = self.client.get(noaa_url).send().await?.error_for_status()?;
        let body = res.text().await?;
        let (_, result) = parse_weather(&body)?;
        Ok(result)
    }

    /// Same function as `get_weather` but a blocking version.
    pub fn get_blocking_weather(&self, station_code: &str) -> Result<WeatherInfo, WeatherError> {
        let noaa_url = format!(
            "https://tgftp.nws.noaa.gov/data/observations/metar/decoded/{}.TXT",
            station_code
        );
        let body = self
            .blocking_client
            .get(noaa_url)
            .send()?
            .error_for_status()?
            .text()?;
        let (_, result) = parse_weather(&body)?;
        Ok(result)
    }
}

// Implementation taken and adapted from
// https://github.com/jaor/xmobar/blob/master/src/Xmobar/Plugins/Monitors/Weather.hs

/// Nom parser for parsing [WeatherInfo] from raw data.
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
    let (i, sky_condition) = parse_sky_condition(i)?;
    let (i, weather) = parse_weather_str(i)?;
    let (i, _) = tag("Temperature:")(i)?;
    let (i, temperature) = parse_temperature(i)?;
    let (i, _) = newline(i)?;
    let (i, _) = tag("Dew Point:")(i)?;
    let (i, dewpoint) = parse_temperature(i)?;
    let (i, _) = newline(i)?;
    let (i, relative_humidity) = parse_relative_humidity(i)?;
    let (i, pressure) = parse_pressure(i)?;
    let winfo = WeatherInfo {
        station,
        weather_time,
        wind,
        visibility: visibility.into(),
        sky_condition,
        weather,
        temperature,
        dewpoint,
        relative_humidity,
        pressure,
    };
    Ok((i, winfo))
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
                if let [c, ..] = country.split('(').collect::<Vec<&str>>()[..] {
                    country = c.trim().to_string();
                }
                Ok(Station {
                    place: s1.to_string(),
                    country,
                })
            }
            _ => Err(format!("Failure parsing {}", i)),
        }
    }
}

impl Default for WindInfo {
    fn default() -> Self {
        WindInfo {
            cardinal: "μ".into(),
            azimuth: 0.0,
            mph: 0.0,
            knots: 0.0,
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
    let (i, pressure) = map_res(take_till(char::is_whitespace), |i: &str| i.parse())(i)?;
    let (i, _) = take_till(|c| c == '\n')(i)?;
    Ok((i, pressure))
}

fn parse_windinfo(i: &str) -> IResult<&str, WindInfo> {
    fn calm_parser(i: &str) -> IResult<&str, WindInfo> {
        let (i, _) = many1(tag("Wind: Calm:0"))(i)?;
        Ok((i, WindInfo::default()))
    }

    fn wind_from_parser(i: &str) -> IResult<&str, WindInfo> {
        let (i, _) = tag("Wind: from the ")(i)?;
        let (i, cardinal) = take_till(char::is_whitespace)(i)?;
        let (i, _) = spaces(i)?;
        let (i, _) = char('(')(i)?;
        let (i, azimuth) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
        let (i, _) = tag(" degrees) at ")(i)?;
        let (i, mph) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
        let (i, _) = tag(" MPH (")(i)?;
        let (i, knots) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
        let (i, _) = take_till(|c| c == '\n')(i)?;
        let wind_info = WindInfo {
            cardinal: cardinal.into(),
            azimuth,
            mph,
            knots,
        };
        Ok((i, wind_info))
    }

    fn wind_var_parser(i: &str) -> IResult<&str, WindInfo> {
        let (i, _) = tag("Wind: Variable at ")(i)?;
        let (i, mph) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
        let (i, _) = tag(" MPH (")(i)?;
        let (i, knots) = map_res(take_till(char::is_whitespace), |s: &str| s.parse())(i)?;
        let (i, _) = take_till(|c| c == '\n')(i)?;
        let wind_info = WindInfo {
            knots,
            mph,
            ..WindInfo::default()
        };
        Ok((i, wind_info))
    }

    alt((calm_parser, wind_from_parser, wind_var_parser))(i)
}

fn parse_sky_condition(i: &str) -> IResult<&str, Option<String>> {
    let (i, sky_tag) = opt(tag("Sky conditions: "))(i)?;
    if sky_tag.is_some() {
        let (i, sky_condition) = take_till(|c| c == '\n')(i)?;
        let (i, _) = newline(i)?;
        Ok((i, Some(sky_condition.to_owned())))
    } else {
        Ok((i, None))
    }
}

fn parse_relative_humidity(i: &str) -> IResult<&str, f64> {
    let (i, _) = tag("Relative Humidity: ")(i)?;
    let (i, humidity) = map_res(take_till(|c| c == '%'), |s: &str| s.parse())(i)?;
    let (i, _) = char('%')(i)?;
    let (i, _) = newline(i)?;
    Ok((i, humidity))
}

fn parse_station(i: &str) -> IResult<&str, Option<Station>> {
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
        celsius,
        fahrenheit,
    };
    Ok((i, temperature))
}

fn parse_time(i: &str) -> IResult<&str, WeatherTime> {
    // Parsers a sample string like this
    // Mar 28, 2021 - 04:00 AM EDT / 2021.03.28 0800 UTC
    let (i, _) = take_till(|c| c == '/')(i)?;
    let (i, _) = char('/')(i)?;
    let (i, _) = char(' ')(i)?;
    let (i, y) = map_res(take_till(|c| c == '.'), |s: &str| s.parse::<u16>())(i)?;
    let (i, _) = char('.')(i)?;
    let (i, m) = map_res(take_till(|c| c == '.'), |s: &str| s.parse::<u8>())(i)?;
    let (i, _) = context("Trying to parse day", char('.'))(i)?;

    let (i, d) = map_res(take_till(|c| c == ' '), |s: &str| s.parse::<u8>())(i)?;
    let (i, _) = char(' ')(i)?;
    let (i, time) = take_till(|c| c == '\n')(i)?;
    Ok((
        i,
        WeatherTime {
            year: y,
            month: m,
            day: d,
            time: time.to_owned(),
        },
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
            azimuth: 0.0,
            mph: 0.0,
            knots: 0.0,
        };
        assert_eq!(parse_windinfo("Wind: Calm:0"), Ok(("", winfo.clone())));
        assert!(parse_windinfo("Wind: unexpected").is_err());

        let china_info = WindInfo {
            cardinal: "NNW".into(),
            azimuth: 340.0,
            mph: 16.0,
            knots: 14.0,
        };

        assert_eq!(
            parse_windinfo("Wind: from the NNW (340 degrees) at 16 MPH (14 KT):0"),
            Ok(("", china_info))
        )
    }

    #[test]
    fn test_temperature() {
        let temp = Temperature {
            fahrenheit: 78.0,
            celsius: 26.0,
        };
        assert_eq!(parse_temperature(" 78 F (26 C)"), Ok(("", temp)));

        let temp = Temperature {
            fahrenheit: 66.0,
            celsius: 19.0,
        };

        assert_eq!(parse_temperature(" 66 F (19 C)"), Ok(("", temp)));
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

        assert_eq!(parse_weather_str(""), Ok(("", None)));

        assert_eq!(
            parse_weather_str("non_existent"),
            Ok(("non_existent", None))
        );
    }

    #[test]
    fn retrieve_test_weather() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
	let app = NoaaApp::new();
        let future = rt.block_on(async { app.get_weather("VOBL".into()).await });
        assert!(future.is_ok());

        let future2 = rt.block_on(async { app.get_weather("non_existent".into()).await });
        assert!(future2.is_err());
    }

    #[test]
    fn retrieve_test_blocking_weather() {
	let app = NoaaApp::new();
        let result = app.get_blocking_weather("VOBL".into());
        assert!(result.is_ok());

        let result2 = app.get_blocking_weather("non_existent".into());
        assert!(result2.is_err());
    }

    #[test]
    fn test_kykm_weather() {
        let weather = r#"YAKIMA AIR TERMINAL, WA, United States (KYKM) 46-34N 120-32W 324M
Dec 30, 2023 - 10:53 PM EST / 2023.12.31 0353 UTC
Wind: Calm:0
Visibility: 5 mile(s):0
Sky conditions: overcast
Weather: mist
Temperature: 42.1 F (5.6 C)
Dew Point: 39.0 F (3.9 C)
Relative Humidity: 88%
Pressure (altimeter): 30.05 in. Hg (1017 hPa)
ob: KYKM 310353Z AUTO 00000KT 5SM BR OVC025 06/04 A3005 RMK AO2 SLP185 T00560039
cycle: 4"#;
        parse_weather(weather).unwrap();
    }

    #[test]
    fn test_vogo_weather() {
        let weather = r#"Station name not available
Dec 30, 2023 - 07:30 AM EST / 2023.12.30 1230 UTC
Wind: from the NNW (340 degrees) at 7 MPH (6 KT):0
Visibility: 3 mile(s):0
Temperature: 84 F (29 C)
Dew Point: 71 F (22 C)
Relative Humidity: 65%
Pressure (altimeter): 29.83 in. Hg (1010 hPa)
ob: VOGO 301230Z 34006KT 6000 NSC 29/22 Q1010 NOSIG
cycle: 12"#;
        parse_weather(weather).unwrap();
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
        let winfo = WeatherInfo {
            station: None,
            weather_time: WeatherTime {
                year: 2021,
                month: 5,
                day: 16,
                time: "1030 UTC".into(),
            },
            wind: WindInfo {
                cardinal: "SSW".into(),
                azimuth: 200.0,
                mph: 12.0,
                knots: 10.0,
            },
            visibility: "4 mile(s):0".into(),
            sky_condition: Some("partly cloudy".to_owned()),
            weather: None,
            temperature: Temperature {
                fahrenheit: 80.0,
                celsius: 27.0,
            },
            dewpoint: Temperature {
                fahrenheit: 66.0,
                celsius: 19.0,
            },
            relative_humidity: 61.0,
            pressure: 1009,
        };

        assert_eq!(parse_weather(weather), Ok(("\nextra", winfo)));
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
        let winfo = WeatherInfo {
            station: Some(Station {
                place: "Qingdao".into(),
                country: "China".into(),
            }),
            weather_time: WeatherTime {
                year: 2021,
                month: 3,
                day: 28,
                time: "0800 UTC".into(),
            },
            wind: WindInfo {
                cardinal: "NNW".into(),
                azimuth: 340.0,
                mph: 16.0,
                knots: 14.0,
            },
            visibility: "1 mile(s):0".into(),
            sky_condition: Some("overcast".to_owned()),
            weather: Some("widespread dust".into()),
            temperature: Temperature {
                fahrenheit: 64.0,
                celsius: 18.0,
            },
            dewpoint: Temperature {
                fahrenheit: 42.0,
                celsius: 6.0,
            },
            relative_humidity: 45.0,
            pressure: 1004,
        };

        assert_eq!(parse_weather(weather), Ok(("", winfo)));

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
        let winfo2 = WeatherInfo {
            station: Some(Station {
                place: "Qingdao".into(),
                country: "China".into(),
            }),
            weather_time: WeatherTime {
                year: 2021,
                month: 3,
                day: 28,
                time: "0800 UTC".into(),
            },
            wind: WindInfo {
                cardinal: "NNW".into(),
                azimuth: 340.0,
                mph: 16.0,
                knots: 14.0,
            },
            visibility: "1 mile(s):0".into(),
            sky_condition: Some("overcast".to_owned()),
            weather: Some("widespread dust".into()),
            temperature: Temperature {
                fahrenheit: 64.0,
                celsius: 18.0,
            },
            dewpoint: Temperature {
                fahrenheit: 42.0,
                celsius: 6.0,
            },
            relative_humidity: 45.0,
            pressure: 1004,
        };

        assert_eq!(parse_weather(weather2), Ok(("\nextra", winfo2)))
    }
}
