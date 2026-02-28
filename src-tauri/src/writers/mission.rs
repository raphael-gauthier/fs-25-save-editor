use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::MissionChange;
use crate::models::mission::MissionStatus;

fn is_mission_tag(tag: &str) -> bool {
    tag.ends_with("Mission") && tag != "missions"
}

pub fn write_mission_changes(
    path: &Path,
    changes: &[MissionChange],
) -> Result<(), AppError> {
    let xml_path = path.join("missions.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<&str, &MissionChange> = changes
        .iter()
        .map(|c| (c.unique_id.as_str(), c))
        .collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    // Track whether we're inside a mission that has a matching change
    let mut active_change: Option<&MissionChange> = None;
    let mut active_mission_tag: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if is_mission_tag(&tag) {
                    let id = attr_str(e, "uniqueId");
                    if let Some(change) = change_map.get(id.as_str()) {
                        active_change = Some(change);
                        active_mission_tag = Some(tag.clone());
                        // Patch status on mission tag if changed
                        let elem = patch_mission_tag(e, &tag, change);
                        write_event(&mut writer, &xml_path, Event::Start(elem))?;
                    } else {
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                } else if active_change.is_some() && tag == "info" {
                    let change = active_change.unwrap();
                    let elem = patch_info(e, change);
                    write_event(&mut writer, &xml_path, Event::Start(elem))?;
                } else {
                    write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if active_change.is_some() && tag == "info" {
                    let change = active_change.unwrap();
                    let elem = patch_info(e, change);
                    write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                } else {
                    write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if Some(&tag) == active_mission_tag.as_ref() {
                    active_change = None;
                    active_mission_tag = None;
                }
                write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                write_event(&mut writer, &xml_path, event.into_owned())?;
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

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn patch_mission_tag(e: &BytesStart, tag_name: &str, change: &MissionChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new(tag_name.to_string());
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "status" if change.status.is_some() => {
                let status = MissionStatus::from_str(change.status.as_ref().unwrap());
                elem.push_attribute(("status", status.to_xml_str()));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    String::from_utf8_lossy(&attr.value).as_ref(),
                ));
            }
        }
    }
    elem
}

fn patch_info(e: &BytesStart, change: &MissionChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("info");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "reward" if change.reward.is_some() => {
                elem.push_attribute(("reward", format!("{:.6}", change.reward.unwrap()).as_str()));
            }
            "completion" if change.completion.is_some() => {
                elem.push_attribute(("completion", format!("{:.6}", change.completion.unwrap()).as_str()));
            }
            "reimbursement" if change.reimbursement.is_some() => {
                elem.push_attribute(("reimbursement", format!("{:.6}", change.reimbursement.unwrap()).as_str()));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    String::from_utf8_lossy(&attr.value).as_ref(),
                ));
            }
        }
    }
    elem
}

fn write_event(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    event: Event<'static>,
) -> Result<(), AppError> {
    writer.write_event(event).map_err(|e| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::mission::parse_missions;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wm_{}", name));
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
    fn test_write_mission_reward() {
        let save = setup_fixture("reward");
        let missions_before = parse_missions(&save).unwrap();
        let harvest = missions_before.iter().find(|m| m.mission_type == "harvest").unwrap();
        let harvest_id = harvest.unique_id.clone();

        let changes = vec![MissionChange {
            unique_id: harvest_id.clone(),
            reward: Some(50000.0),
            completion: None,
            status: None,
            reimbursement: None,
        }];
        write_mission_changes(&save, &changes).unwrap();
        let missions = parse_missions(&save).unwrap();
        let m = missions.iter().find(|m| m.unique_id == harvest_id).unwrap();
        assert!((m.reward - 50000.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_mission_completion() {
        let save = setup_fixture("completion");
        let missions_before = parse_missions(&save).unwrap();
        let harvest = missions_before.iter().find(|m| m.mission_type == "harvest").unwrap();
        let harvest_id = harvest.unique_id.clone();

        let changes = vec![MissionChange {
            unique_id: harvest_id.clone(),
            reward: None,
            completion: Some(1.0),
            status: Some("COMPLETED".to_string()),
            reimbursement: None,
        }];
        write_mission_changes(&save, &changes).unwrap();
        let missions = parse_missions(&save).unwrap();
        let m = missions.iter().find(|m| m.unique_id == harvest_id).unwrap();
        assert!((m.completion - 1.0).abs() < 0.01);
        assert_eq!(m.status, MissionStatus::Completed);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_mission_roundtrip() {
        let save = setup_fixture("roundtrip_m");
        let before = parse_missions(&save).unwrap();
        let harvest = before.iter().find(|m| m.mission_type == "harvest").unwrap();
        let harvest_id = harvest.unique_id.clone();
        let plow = before.iter().find(|m| m.mission_type == "plow").unwrap();
        let plow_id = plow.unique_id.clone();

        let changes = vec![MissionChange {
            unique_id: harvest_id.clone(),
            reward: Some(99999.0),
            completion: Some(0.75),
            status: None,
            reimbursement: Some(5000.0),
        }];
        write_mission_changes(&save, &changes).unwrap();
        let after = parse_missions(&save).unwrap();

        assert_eq!(after.len(), before.len());
        let m = after.iter().find(|m| m.unique_id == harvest_id).unwrap();
        assert!((m.reward - 99999.0).abs() < 0.01);
        assert!((m.completion - 0.75).abs() < 0.01);
        assert!((m.reimbursement - 5000.0).abs() < 0.01);

        // Other missions untouched
        let plow_before = before.iter().find(|m| m.unique_id == plow_id).unwrap();
        let plow_after = after.iter().find(|m| m.unique_id == plow_id).unwrap();
        assert!((plow_before.reward - plow_after.reward).abs() < 0.01);

        let _ = std::fs::remove_dir_all(&save);
    }
}
