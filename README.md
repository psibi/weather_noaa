# weathernoaa

[![CI](https://github.com/psibi/weather_noaa/actions/workflows/ci.yml/badge.svg)](https://github.com/psibi/weather_noaa/actions)

API wrapper over NOAA's observatory data to find weather
information. For finding the weather information, you need to know the
name of the station code which can be obtained from [here](https://www.weather.gov/arh/stationlist). In
general, figuring out station IDs is harder. These are the various
resources I usually use (if you find any source, please send a PR):

- [METAR Observation Station Identifiers](https://www.cnrfc.noaa.gov/metar.php)
- [India METAR Station ID](https://amssdelhi.gov.in/Palam1.php)

You can find all the [supported station id](https://tgftp.nws.noaa.gov/data/observations/metar/stations/) here. Alternatively,
you can also use the executable in the repository to confirm that it
works:

``` shellsession
❯ cargo run --bin noaa info --station-id VOBL
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/noaa info --station-id VOBL`
WeatherInfo {
    station: None,
    weather_time: WeatherTime {
        year: 2023,
        month: 12,
        day: 30,
        time: "1330 UTC",
    },
    wind: WindInfo {
        cardinal: "E",
        azimuth: "080",
        mph: "9",
        knots: "8",
    },
    visibility: "greater than 7 mile(s):0",
    sky_condition: "mostly clear",
    weather: None,
    temperature: Temperature {
        celsius: 23,
        fahrenheit: 73,
    },
    dewpoint: Temperature {
        celsius: 14,
        fahrenheit: 57,
    },
    relative_humidity: "56%",
    pressure: 1017,
}
```


## API Usage

``` rust
use anyhow::Result;
use weathernoaa::weather::*;

#[tokio::main]
async fn main() -> Result<()> {
    let result = get_weather("VOBL".into()).await?;
    println!("{:#?}", result);
    Ok(())
}
```

Running it will give this:

``` rust
WeatherInfo {
 station: None,
  weather_time:
   WeatherTime {
    year: 2021,
    month: 5,
    day: 16,
    time: "1200 UTC",
   },
  wind:
   WindInfo {
    cardinal: "SSW",
    azimuth: "210",
    mph: "10",
    knots: "9",
   },
  visibility: "4 mile(s):0",
  sky_condition: "partly cloudy",
  weather: Some("light drizzle"),
  temperature: Temperature {
    celsius: 26,
    fahrenheit: 78,
  },
  dewpoint: Temperature {
    celsius: 19,
    fahrenheit: 66,
  },
  relative_humidity: "65%",
  pressure: 1010,
};
```
