# weathernoaa

API wrapper over NOAA's observatory data to find weather
information. For finding the weather information, you need to know the
name of the station code which can be obtained from
[here](https://www.ncdc.noaa.gov/data-access/land-based-station-data/station-metadata).

## Usage

``` shellsession
use anyhow::Result;
use weathernoaa::weather::*;

#[tokio::main]
async fn main() -> Result<()> {
    let result = get_weather("VOBL".into()).await?;
    println!("{:?}", result);
    Ok(())
}
```

Running it will give this:

```
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
