use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::environment::{Environment, WeatherEvent};

fn text_content(reader: &mut Reader<&[u8]>) -> String {
    let mut buf = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(ref e)) => {
                buf.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(_)) | Ok(Event::Eof) => break,
            _ => {}
        }
    }
    buf.trim().to_string()
}

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn attr_f64(e: &quick_xml::events::BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

/// Parse environment.xml and return the Environment data.
pub fn parse_environment(path: &Path) -> Result<Environment, AppError> {
    let xml_path = path.join("environment.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);

    let mut day_time: f64 = 0.0;
    let mut current_day: u32 = 0;
    let mut current_monotonic_day: u32 = 0;
    let mut days_per_period: u8 = 1;
    let mut weather_forecast: Vec<WeatherEvent> = Vec::new();
    let mut snow_height: f64 = 0.0;
    let mut ground_wetness: f64 = 0.0;
    let mut in_forecast = false;
    let mut in_weather = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "dayTime" => {
                        day_time = text_content(&mut reader).parse().unwrap_or(0.0);
                    }
                    "currentDay" => {
                        current_day = text_content(&mut reader).parse().unwrap_or(0);
                    }
                    "currentMonotonicDay" => {
                        current_monotonic_day = text_content(&mut reader).parse().unwrap_or(0);
                    }
                    "daysPerPeriod" => {
                        days_per_period = text_content(&mut reader).parse().unwrap_or(1);
                    }
                    "weather" => {
                        in_weather = true;
                    }
                    "forecast" => {
                        in_forecast = true;
                    }
                    "instance" if in_forecast => {
                        weather_forecast.push(WeatherEvent {
                            type_name: attr_str(e, "typeName"),
                            season: attr_str(e, "season"),
                            variation_index: attr_str(e, "variationIndex").parse().unwrap_or(0),
                            start_day: attr_str(e, "startDay").parse().unwrap_or(0),
                            start_day_time: attr_str(e, "startDayTime").parse().unwrap_or(0),
                            duration: attr_str(e, "duration").parse().unwrap_or(0),
                        });
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "instance" if in_forecast => {
                        weather_forecast.push(WeatherEvent {
                            type_name: attr_str(e, "typeName"),
                            season: attr_str(e, "season"),
                            variation_index: attr_str(e, "variationIndex").parse().unwrap_or(0),
                            start_day: attr_str(e, "startDay").parse().unwrap_or(0),
                            start_day_time: attr_str(e, "startDayTime").parse().unwrap_or(0),
                            duration: attr_str(e, "duration").parse().unwrap_or(0),
                        });
                    }
                    "snow" if in_weather => {
                        snow_height = attr_f64(e, "height");
                    }
                    "ground" if in_weather => {
                        ground_wetness = attr_f64(e, "wetness");
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "forecast" => in_forecast = false,
                    "weather" => in_weather = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
            _ => {}
        }
    }

    Ok(Environment {
        day_time,
        current_day,
        current_monotonic_day,
        days_per_period,
        weather_forecast,
        snow_height,
        ground_wetness,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    #[test]
    fn test_parse_environment_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let env = parse_environment(&path).unwrap();
        assert!((env.day_time - 43200.0).abs() < 0.01);
        assert_eq!(env.current_day, 54);
        assert_eq!(env.current_monotonic_day, 54);
        assert_eq!(env.days_per_period, 3);
        assert_eq!(env.weather_forecast.len(), 4);
        assert_eq!(env.weather_forecast[0].type_name, "SUN");
        assert_eq!(env.weather_forecast[0].season, "SUMMER");
        assert_eq!(env.weather_forecast[0].start_day, 54);
        assert_eq!(env.weather_forecast[0].start_day_time, 10800000);
        assert_eq!(env.weather_forecast[0].duration, 36000000);
        assert_eq!(env.weather_forecast[3].type_name, "TWISTER");
        assert!((env.snow_height - 0.5).abs() < 0.01);
        assert!((env.ground_wetness - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_parse_environment_empty_forecast() {
        let dir = std::env::temp_dir().join("fs25_test_env_empty_forecast");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<environment>
    <dayTime>100.0</dayTime>
    <currentDay>1</currentDay>
    <currentMonotonicDay>1</currentMonotonicDay>
    <daysPerPeriod>1</daysPerPeriod>
    <weather timeSinceLastRain="0">
        <forecast/>
        <snow height="0.000000"/>
        <ground wetness="0.000000"/>
    </weather>
</environment>"#;
        std::fs::write(dir.join("environment.xml"), xml).unwrap();
        let env = parse_environment(&dir).unwrap();
        assert!((env.day_time - 100.0).abs() < 0.01);
        assert_eq!(env.current_day, 1);
        assert!(env.weather_forecast.is_empty());
        assert!((env.snow_height).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_environment_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_environment");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_environment(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
