use std::path::Path;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::EnvironmentChanges;
use crate::models::environment::WeatherEvent;

/// Applies environment changes to environment.xml.
///
/// Scalar fields (dayTime, currentDay, snow height, ground wetness) are patched in place.
/// If `weather_forecast` is set, the entire `<forecast>` section is replaced.
pub fn write_environment_changes(
    path: &Path,
    changes: &EnvironmentChanges,
) -> Result<(), AppError> {
    let xml_path = path.join("environment.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    let mut in_weather = false;
    let mut in_forecast = false;
    let mut skip_until_forecast_end = false;
    let mut forecast_written = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "dayTime" => {
                        write_ev(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        // Read original text
                        let original_text = read_text_content(&mut reader);
                        if let Some(val) = changes.day_time {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Text(BytesText::new(&format!("{:.6}", val)).into_owned()),
                            )?;
                        } else {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Text(BytesText::new(&original_text).into_owned()),
                            )?;
                        }
                        write_ev(
                            &mut writer,
                            &xml_path,
                            Event::End(BytesEnd::new("dayTime").into_owned()),
                        )?;
                        continue;
                    }
                    "currentDay" => {
                        write_ev(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        let original_text = read_text_content(&mut reader);
                        if let Some(val) = changes.current_day {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Text(BytesText::new(&val.to_string()).into_owned()),
                            )?;
                        } else {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Text(BytesText::new(&original_text).into_owned()),
                            )?;
                        }
                        write_ev(
                            &mut writer,
                            &xml_path,
                            Event::End(BytesEnd::new("currentDay").into_owned()),
                        )?;
                        continue;
                    }
                    "weather" => {
                        in_weather = true;
                        write_ev(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        continue;
                    }
                    "forecast" if in_weather => {
                        in_forecast = true;
                        if changes.weather_forecast.is_some() {
                            // Skip original forecast content, we'll write our own
                            skip_until_forecast_end = true;
                            continue;
                        } else {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Start(e.clone().into_owned()),
                            )?;
                            continue;
                        }
                    }
                    _ => {
                        if skip_until_forecast_end {
                            continue;
                        }
                        write_ev(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if skip_until_forecast_end {
                    continue;
                }
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "snow" if in_weather && !in_forecast => {
                        if let Some(val) = changes.snow_height {
                            let mut elem = BytesStart::new("snow");
                            elem.push_attribute(("height", format!("{:.6}", val).as_str()));
                            write_ev(&mut writer, &xml_path, Event::Empty(elem))?;
                        } else {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Empty(e.clone().into_owned()),
                            )?;
                        }
                    }
                    "ground" if in_weather && !in_forecast => {
                        if let Some(val) = changes.ground_wetness {
                            let mut elem = BytesStart::new("ground");
                            elem.push_attribute(("wetness", format!("{:.6}", val).as_str()));
                            write_ev(&mut writer, &xml_path, Event::Empty(elem))?;
                        } else {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Empty(e.clone().into_owned()),
                            )?;
                        }
                    }
                    "forecast" if in_weather => {
                        // Self-closing <forecast/> (empty forecast)
                        if let Some(ref events) = changes.weather_forecast {
                            write_forecast_section(&mut writer, &xml_path, events)?;
                            forecast_written = true;
                        } else {
                            write_ev(
                                &mut writer,
                                &xml_path,
                                Event::Empty(e.clone().into_owned()),
                            )?;
                        }
                    }
                    _ => {
                        write_ev(
                            &mut writer,
                            &xml_path,
                            Event::Empty(e.clone().into_owned()),
                        )?;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "forecast" if in_weather => {
                        in_forecast = false;
                        if skip_until_forecast_end {
                            skip_until_forecast_end = false;
                            // Write the replacement forecast
                            if let Some(ref events) = changes.weather_forecast {
                                if !forecast_written {
                                    write_forecast_section(&mut writer, &xml_path, events)?;
                                }
                            }
                            continue;
                        }
                        write_ev(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                    }
                    "weather" => {
                        in_weather = false;
                        write_ev(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                    }
                    _ => {
                        if skip_until_forecast_end {
                            continue;
                        }
                        write_ev(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                if skip_until_forecast_end {
                    continue;
                }
                write_ev(&mut writer, &xml_path, event.into_owned())?;
            }
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

/// Writes a complete `<forecast>...</forecast>` section with the given events.
fn write_forecast_section(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    events: &[WeatherEvent],
) -> Result<(), AppError> {
    write_ev(
        writer,
        xml_path,
        Event::Start(BytesStart::new("forecast")),
    )?;
    // Newline after <forecast>
    write_ev(
        writer,
        xml_path,
        Event::Text(BytesText::new("\n").into_owned()),
    )?;

    for event in events {
        let mut elem = BytesStart::new("instance");
        elem.push_attribute(("typeName", event.type_name.as_str()));
        elem.push_attribute(("season", event.season.as_str()));
        elem.push_attribute((
            "variationIndex",
            event.variation_index.to_string().as_str(),
        ));
        elem.push_attribute(("startDay", event.start_day.to_string().as_str()));
        elem.push_attribute(("startDayTime", event.start_day_time.to_string().as_str()));
        elem.push_attribute(("duration", event.duration.to_string().as_str()));
        write_ev(
            writer,
            xml_path,
            Event::Text(BytesText::new("            ").into_owned()),
        )?;
        write_ev(writer, xml_path, Event::Empty(elem))?;
        write_ev(
            writer,
            xml_path,
            Event::Text(BytesText::new("\n").into_owned()),
        )?;
    }

    write_ev(
        writer,
        xml_path,
        Event::Text(BytesText::new("        ").into_owned()),
    )?;
    write_ev(
        writer,
        xml_path,
        Event::End(BytesEnd::new("forecast").into_owned()),
    )?;

    Ok(())
}

fn read_text_content(reader: &mut Reader<&[u8]>) -> String {
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
    buf
}

fn write_ev(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    event: Event<'static>,
) -> Result<(), AppError> {
    writer
        .write_event(event)
        .map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::environment::parse_environment;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_we_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        for entry in std::fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
            }
        }
        dst
    }

    #[test]
    fn test_write_environment_day_time() {
        let save = setup_fixture("env_daytime");
        let changes = EnvironmentChanges {
            day_time: Some(72000.0),
            current_day: Some(100),
            snow_height: None,
            ground_wetness: None,
            weather_forecast: None,
        };
        write_environment_changes(&save, &changes).unwrap();
        let env = parse_environment(&save).unwrap();
        assert!((env.day_time - 72000.0).abs() < 0.01);
        assert_eq!(env.current_day, 100);
        // Forecast should be preserved
        assert_eq!(env.weather_forecast.len(), 4);
        assert_eq!(env.weather_forecast[0].type_name, "SUN");
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_environment_snow_wetness() {
        let save = setup_fixture("env_snow");
        let changes = EnvironmentChanges {
            day_time: None,
            current_day: None,
            snow_height: Some(1.5),
            ground_wetness: Some(0.8),
            weather_forecast: None,
        };
        write_environment_changes(&save, &changes).unwrap();
        let env = parse_environment(&save).unwrap();
        assert!((env.snow_height - 1.5).abs() < 0.01);
        assert!((env.ground_wetness - 0.8).abs() < 0.01);
        // day_time should be preserved
        assert!((env.day_time - 43200.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_environment_weather_forecast() {
        let save = setup_fixture("env_forecast");
        let new_forecast = vec![
            WeatherEvent {
                type_name: "SUN".to_string(),
                season: "SUMMER".to_string(),
                variation_index: 1,
                start_day: 54,
                start_day_time: 0,
                duration: 86400000,
            },
            WeatherEvent {
                type_name: "RAIN".to_string(),
                season: "SUMMER".to_string(),
                variation_index: 2,
                start_day: 55,
                start_day_time: 0,
                duration: 43200000,
            },
        ];
        let changes = EnvironmentChanges {
            day_time: None,
            current_day: None,
            snow_height: None,
            ground_wetness: None,
            weather_forecast: Some(new_forecast),
        };
        write_environment_changes(&save, &changes).unwrap();
        let env = parse_environment(&save).unwrap();
        assert_eq!(env.weather_forecast.len(), 2);
        assert_eq!(env.weather_forecast[0].type_name, "SUN");
        assert_eq!(env.weather_forecast[0].duration, 86400000);
        assert_eq!(env.weather_forecast[1].type_name, "RAIN");
        assert_eq!(env.weather_forecast[1].start_day, 55);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_environment_roundtrip() {
        let save = setup_fixture("env_roundtrip");
        let new_forecast = vec![WeatherEvent {
            type_name: "CLOUDY".to_string(),
            season: "WINTER".to_string(),
            variation_index: 3,
            start_day: 60,
            start_day_time: 7200000,
            duration: 18000000,
        }];
        let changes = EnvironmentChanges {
            day_time: Some(10000.0),
            current_day: Some(60),
            snow_height: Some(2.0),
            ground_wetness: Some(0.9),
            weather_forecast: Some(new_forecast),
        };
        write_environment_changes(&save, &changes).unwrap();
        let env = parse_environment(&save).unwrap();
        assert!((env.day_time - 10000.0).abs() < 0.01);
        assert_eq!(env.current_day, 60);
        assert!((env.snow_height - 2.0).abs() < 0.01);
        assert!((env.ground_wetness - 0.9).abs() < 0.01);
        assert_eq!(env.weather_forecast.len(), 1);
        assert_eq!(env.weather_forecast[0].type_name, "CLOUDY");
        assert_eq!(env.weather_forecast[0].season, "WINTER");
        let _ = std::fs::remove_dir_all(&save);
    }
}
