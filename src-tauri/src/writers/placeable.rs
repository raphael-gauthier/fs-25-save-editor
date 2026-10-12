use std::path::Path;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::writers::replace_xml_file;
use crate::models::changes::{ObjectCopy, ObjectStorageChange, PlaceableChange, ProductionStockChange, SiloStockChange};
use crate::models::placeable::SiloStorageKind;

/// Applies a list of placeable changes to placeables.xml.
/// Patch strategy: reads original XML, modifies only targeted attributes/elements, rewrites atomically.
pub fn write_placeable_changes(
    path: &Path,
    changes: &[PlaceableChange],
) -> Result<(), AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<usize, &PlaceableChange> = changes
        .iter()
        .map(|c| (c.index, c))
        .collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    // State tracking
    let mut placeable_index: usize = 0;
    let mut in_placeable = false;
    let mut current_change: Option<&PlaceableChange> = None;
    // `<storage>`/`<node>` are shared by silos, husbandries and production points,
    // so stock patching is gated on being inside a `<productionPoint>`.
    let mut in_constructible = false;
    let mut in_production_point = false;
    let mut in_production_storage = false;
    let mut in_silo = false;
    // The silo `<storage>` or `<siloExtension>` open around the current event,
    // with the fill types whose `<node>` was already written
    let mut open_silo: Option<(SiloStorageKind, u32, Vec<String>)> = None;
    // Indentation before the last element, reused for added `<node>`s
    let mut last_indent = String::from("\n");

    loop {
        match reader.read_event() {
            Ok(Event::Text(ref t)) => {
                if t.trim().is_empty() {
                    last_indent = t.to_string();
                }
                write_event(&mut writer, &xml_path, Event::Text(t.clone().into_owned()))?;
            }
            Ok(Event::Start(ref e)) => {
                let tag = e.name().as_ref().to_string();

                match tag.as_str() {
                    "silo" if in_placeable => {
                        in_silo = true;
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                    "storage" | "siloExtension" if in_placeable && (in_silo || tag == "siloExtension") => {
                        open_silo = Some((silo_kind(in_silo), silo_index(e, in_silo), Vec::new()));
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                    "placeable" if !in_placeable => {
                        in_placeable = true;
                        current_change = change_map.get(&placeable_index).copied();
                        placeable_index += 1;

                        if let Some(change) = current_change {
                            let elem = patch_placeable_start(e, change);
                            write_event(&mut writer, &xml_path, Event::Start(elem))?;
                        } else {
                            write_event(
                                &mut writer,
                                &xml_path,
                                Event::Start(e.clone().into_owned()),
                            )?;
                        }
                    }
                    "constructible" if in_placeable => {
                        in_constructible = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    "productionPoint" if in_placeable => {
                        in_production_point = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    "storage" if in_production_point => {
                        in_production_storage = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    _ => {
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = e.name().as_ref().to_string();

                // A self-closing <placeable .../> (pre-placed buildings, ~37% of a real
                // save) never opens a Start/End pair, so it never sets `in_placeable`.
                // It must still be counted here — otherwise placeable_index drifts out
                // of sync with the parser and every later edit lands on the wrong
                // building. It has no children, so only its own attributes are patched.
                if tag == "placeable" {
                    let change = change_map.get(&placeable_index).copied();
                    placeable_index += 1;

                    match change {
                        Some(change) => {
                            let elem = patch_placeable_start(e, change);
                            write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                        }
                        None => {
                            write_event(
                                &mut writer,
                                &xml_path,
                                Event::Empty(e.clone().into_owned()),
                            )?;
                        }
                    }
                    continue;
                }

                // A silo storage without any fill level is a self-closing element
                if (tag == "storage" && in_silo) || tag == "siloExtension" {
                    let key = (silo_kind(in_silo), silo_index(e, in_silo));
                    let added = silo_additions(current_change, key, &[]);
                    if added.is_empty() {
                        write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                    } else {
                        let indent = last_indent.clone();
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        for node in added {
                            write_indent(&mut writer, &xml_path, &format!("{}    ", indent))?;
                            write_event(&mut writer, &xml_path, Event::Empty(node))?;
                        }
                        write_indent(&mut writer, &xml_path, &indent)?;
                        write_event(&mut writer, &xml_path, Event::End(BytesEnd::new(tag)))?;
                    }
                    continue;
                }

                if tag == "node" {
                    if let Some((kind, index, ref mut written)) = open_silo {
                        let fill_type = attr_str(e, "fillType");
                        let stock = current_change
                            .and_then(|c| c.silo_stocks.as_ref())
                            .and_then(|stocks| {
                                stocks.iter().find(|s| s.kind == kind && s.index == index && s.fill_type == fill_type)
                            });
                        written.push(fill_type);
                        match stock {
                            // Only fill levels above 0 are saved, as the game does
                            Some(stock) if stock.amount <= 0.0 => {}
                            Some(stock) => {
                                let elem = patch_fill_level(e, stock.amount);
                                write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                            }
                            None => {
                                write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                            }
                        }
                        continue;
                    }
                }

                if let Some(change) = current_change {
                    match tag.as_str() {
                        "input" if in_constructible && change.complete_construction => {
                            let elem = patch_construction_input_complete(e);
                            write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                            continue;
                        }
                        "node" if in_production_storage => {
                            if let Some(ref stock_changes) = change.production_storage {
                                let fill_type = attr_str(e, "fillType");
                                if let Some(sc) =
                                    stock_changes.iter().find(|s| s.fill_type == fill_type)
                                {
                                    let elem = patch_storage_node(e, sc);
                                    write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                write_event(
                    &mut writer,
                    &xml_path,
                    Event::Empty(e.clone().into_owned()),
                )?;
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name().as_ref().to_string();

                if matches!(tag.as_str(), "storage" | "siloExtension") {
                    if let Some((kind, index, written)) = open_silo.take() {
                        let indent = last_indent.clone();
                        for node in silo_additions(current_change, (kind, index), &written) {
                            write_indent(&mut writer, &xml_path, "    ")?;
                            write_event(&mut writer, &xml_path, Event::Empty(node))?;
                            write_indent(&mut writer, &xml_path, &indent)?;
                        }
                    }
                }

                match tag.as_str() {
                    "placeable" if in_placeable => {
                        in_placeable = false;
                        current_change = None;
                        in_constructible = false;
                        in_production_point = false;
                        in_production_storage = false;
                        in_silo = false;
                        open_silo = None;
                    }
                    "silo" => in_silo = false,
                    "constructible" => in_constructible = false,
                    "productionPoint" => {
                        in_production_point = false;
                        in_production_storage = false;
                    }
                    "storage" if in_production_point => in_production_storage = false,
                    _ => {}
                }

                write_event(
                    &mut writer,
                    &xml_path,
                    Event::End(e.clone().into_owned()),
                )?;
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

    let mut output = writer.into_inner();
    if changes.iter().any(|c| c.object_storage.is_some()) {
        let first_pass = String::from_utf8(output).map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;
        output = apply_object_storage_changes(&xml_path, &first_pass, &change_map)?;
    }
    replace_xml_file(&xml_path, &content, &output)?;

    Ok(())
}

/// Fermentation time written to finish a bale's fermentation: far beyond any
/// bale's fermenting time (days x economic difficulty), so the game's
/// BaleManager ends it, with the bale's own output fill type, on its next update.
const FERMENTATION_DONE_TIME: f64 = 1e12;

/// Second pass over placeables.xml for object storage changes: removes
/// objects, copies them (children included) after the last one, and
/// finishes fermentations.
fn apply_object_storage_changes(
    xml_path: &Path,
    content: &str,
    change_map: &std::collections::HashMap<usize, &PlaceableChange>,
) -> Result<Vec<u8>, AppError> {
    let mut reader = Reader::from_str(content);
    let mut writer = Writer::new(Vec::new());
    let mut placeable_index = 0usize;
    let mut change: Option<&ObjectStorageChange> = None;
    let mut in_storage = false;
    let mut object_index = 0usize;
    // Depth inside the current <object>, 0 between objects
    let mut object_depth = 0usize;
    // Whether the current <object> is removed: still captured, not written
    let mut removing = false;
    let mut capture: Option<(usize, Vec<Event<'static>>)> = None;
    let mut captured: Vec<(usize, Vec<Event<'static>>)> = Vec::new();
    let mut last_indent = String::from("\n");

    loop {
        let event = reader.read_event().map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;
        if let Event::Eof = event {
            break;
        }

        let mut event = event.into_owned();
        match &event {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == "placeable" => {
                change = change_map.get(&placeable_index).and_then(|c| c.object_storage.as_ref());
                placeable_index += 1;
            }
            Event::Start(e) if e.name().as_ref() == "objectStorage" && change.is_some() => {
                in_storage = true;
                object_index = 0;
                captured.clear();
            }
            Event::Start(e) | Event::Empty(e)
                if in_storage && object_depth == 0 && e.name().as_ref() == "object" =>
            {
                let index = object_index;
                object_index += 1;
                removing = change.is_some_and(|c| c.remove.contains(&index));
                if change.is_some_and(|c| c.copies.iter().any(|copy| copy.index == index)) {
                    capture = Some((index, Vec::new()));
                }
                if matches!(event, Event::Start(_)) {
                    object_depth = 1;
                }
            }
            Event::Start(_) if object_depth > 0 => object_depth += 1,
            Event::End(_) if object_depth > 0 => object_depth -= 1,
            Event::Empty(e)
                if object_depth > 0
                    && e.name().as_ref() == "fermentation"
                    && change.is_some_and(|c| c.finish_fermentation)
                    && attr_str(e, "isFermenting") == "true" =>
            {
                event = Event::Empty(patch_fermentation_time(e));
            }
            Event::Text(t) if t.trim().is_empty() => last_indent = t.to_string(),
            Event::End(e) if in_storage && e.name().as_ref() == "objectStorage" => {
                in_storage = false;
                let indent = last_indent.clone();
                for copy in change.map(|c| c.copies.as_slice()).unwrap_or_default() {
                    let Some((_, events)) = captured.iter().find(|(i, _)| *i == copy.index) else { continue };
                    for _ in 0..copy.count {
                        write_indent(&mut writer, xml_path, "    ")?;
                        for copied in events {
                            write_event(&mut writer, xml_path, copied.clone())?;
                        }
                        write_indent(&mut writer, xml_path, &indent)?;
                    }
                }
            }
            _ => {}
        }

        if let Some((_, ref mut events)) = capture {
            events.push(event.clone());
            if object_depth == 0 {
                captured.push(capture.take().unwrap());
            }
        }
        if removing {
            removing = object_depth > 0;
            continue;
        }
        write_event(&mut writer, xml_path, event)?;
    }

    Ok(writer.into_inner())
}

fn patch_fermentation_time(e: &BytesStart) -> BytesStart<'static> {
    let mut elem = BytesStart::new("fermentation");
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() != "time" {
            elem.push_attribute(attr);
        }
    }
    elem.push_attribute(("time", format!("{:.6}", FERMENTATION_DONE_TIME).as_str()));
    elem
}

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
        .unwrap_or_default()
}

fn patch_placeable_start(e: &BytesStart, change: &PlaceableChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("placeable");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "farmId" if change.farm_id.is_some() => {
                elem.push_attribute((
                    "farmId",
                    change.farm_id.unwrap().to_string().as_str(),
                ));
            }
            "price" if change.price.is_some() => {
                elem.push_attribute((
                    "price",
                    format!("{:.6}", change.price.unwrap()).as_str(),
                ));
            }
            _ => {
                elem.push_attribute(attr);
            }
        }
    }
    elem
}

/// Marks a construction material as fully delivered by zeroing its outstanding amount.
fn patch_construction_input_complete(e: &BytesStart) -> BytesStart<'static> {
    let mut elem = BytesStart::new("input");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "remainingAmount" => {
                elem.push_attribute(("remainingAmount", "0.000000"));
            }
            _ => {
                elem.push_attribute(attr);
            }
        }
    }
    elem
}

fn patch_storage_node(e: &BytesStart, change: &ProductionStockChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("node");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "fillLevel" => {
                elem.push_attribute((
                    "fillLevel",
                    format!("{:.6}", change.amount).as_str(),
                ));
            }
            _ => {
                elem.push_attribute(attr);
            }
        }
    }
    elem
}

fn silo_kind(in_silo: bool) -> SiloStorageKind {
    if in_silo {
        SiloStorageKind::Silo
    } else {
        SiloStorageKind::Extension
    }
}

/// `<storage index>` of a silo storage, 0 for an extension.
fn silo_index(e: &BytesStart, in_silo: bool) -> u32 {
    if in_silo {
        attr_str(e, "index").parse().unwrap_or(0)
    } else {
        0
    }
}

/// `<node>`s to add to silo storage `key`: changes above 0 for fill types it
/// did not already hold (`written`).
fn silo_additions(
    change: Option<&PlaceableChange>,
    key: (SiloStorageKind, u32),
    written: &[String],
) -> Vec<BytesStart<'static>> {
    let Some(stocks) = change.and_then(|c| c.silo_stocks.as_ref()) else {
        return Vec::new();
    };
    stocks
        .iter()
        .filter(|s| (s.kind, s.index) == key && s.amount > 0.0 && !written.contains(&s.fill_type))
        .map(|s| {
            let mut node = BytesStart::new("node");
            node.push_attribute(("fillType", s.fill_type.as_str()));
            node.push_attribute(("fillLevel", format!("{:.6}", s.amount).as_str()));
            node
        })
        .collect()
}

fn patch_fill_level(e: &BytesStart, amount: f64) -> BytesStart<'static> {
    let mut elem = BytesStart::new("node");
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == "fillLevel" {
            elem.push_attribute(("fillLevel", format!("{:.6}", amount).as_str()));
        } else {
            elem.push_attribute(attr);
        }
    }
    elem
}

fn write_indent(writer: &mut Writer<Vec<u8>>, xml_path: &Path, indent: &str) -> Result<(), AppError> {
    write_event(writer, xml_path, Event::Text(BytesText::from_escaped(indent).into_owned()))
}

fn write_event(
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
    use crate::parsers::placeable::parse_placeables;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wp_{}", name));
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

    fn reference_copy(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("savegames")
            .join("placeables.xml");
        let dst = std::env::temp_dir().join(format!("fs25_test_wp_ref_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        std::fs::copy(src, dst.join("placeables.xml")).unwrap();
        dst
    }

    fn silo_change(index: usize, stocks: Vec<SiloStockChange>) -> PlaceableChange {
        PlaceableChange {
            index,
            farm_id: None,
            price: None,
            complete_construction: false,
            production_storage: None,
            silo_stocks: Some(stocks),
            object_storage: None,
        }
    }

    fn set(kind: SiloStorageKind, fill_type: &str, amount: f64) -> SiloStockChange {
        SiloStockChange { kind, index: if kind == SiloStorageKind::Silo { 1 } else { 0 }, fill_type: fill_type.into(), amount }
    }

    fn levels(save: &std::path::Path, index: usize, kind: SiloStorageKind) -> Vec<(String, f64)> {
        let placeables = parse_placeables(save).unwrap();
        let storage = placeables[index].silo_storages.iter().find(|s| s.kind == kind).unwrap();
        storage.stocks.iter().map(|s| (s.fill_type.clone(), s.amount)).collect()
    }

    /// Index of the reference save's placeable whose silo storage matches `pred`.
    fn silo_placeable(save: &std::path::Path, pred: impl Fn(&crate::models::placeable::SiloStorage) -> bool) -> usize {
        parse_placeables(save).unwrap().iter().position(|p| p.silo_storages.iter().any(&pred)).unwrap()
    }

    #[test]
    fn test_write_silo_levels_set_add_and_remove() {
        let save = reference_copy("silo_levels");
        let farm_silo = silo_placeable(&save, |s| s.stocks.len() == 2);
        let changes = vec![silo_change(farm_silo, vec![
            set(SiloStorageKind::Silo, "WHEAT", 150000.0),
            set(SiloStorageKind::Silo, "SUNFLOWER", 0.0),
            set(SiloStorageKind::Silo, "OAT", 1234.5),
        ])];

        write_placeable_changes(&save, &changes).unwrap();

        assert_eq!(levels(&save, farm_silo, SiloStorageKind::Silo), vec![("WHEAT".into(), 150000.0), ("OAT".into(), 1234.5)]);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_silo_fills_an_empty_storage_element() {
        let save = reference_copy("silo_empty");
        let empty = silo_placeable(&save, |s| s.stocks.is_empty() && s.farm_id == 1);
        let changes = vec![silo_change(empty, vec![set(SiloStorageKind::Silo, "BARLEY", 5000.0)])];

        write_placeable_changes(&save, &changes).unwrap();

        assert_eq!(levels(&save, empty, SiloStorageKind::Silo), vec![("BARLEY".into(), 5000.0)]);
        let xml = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(xml.contains(r#"<node fillType="BARLEY" fillLevel="5000.000000"/>"#));
        // Added lines reuse the file's own line endings, unescaped
        assert!(!xml.contains("&#13;") && !xml.contains("&#10;"));
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_silo_extension_levels() {
        let save = reference_copy("silo_extension");
        let extension = silo_placeable(&save, |s| s.kind == SiloStorageKind::Extension);
        let changes = vec![silo_change(extension, vec![
            set(SiloStorageKind::Extension, "WHEAT", 1000.0),
            set(SiloStorageKind::Extension, "CANOLA", 2000.0),
        ])];

        write_placeable_changes(&save, &changes).unwrap();

        assert_eq!(
            levels(&save, extension, SiloStorageKind::Extension),
            vec![("WHEAT".into(), 1000.0), ("CANOLA".into(), 2000.0)]
        );
        // Other silos are untouched
        let farm_silo = silo_placeable(&save, |s| s.stocks.len() == 2);
        assert_eq!(levels(&save, farm_silo, SiloStorageKind::Silo)[0], ("WHEAT".into(), 20657.392578));
        let _ = std::fs::remove_dir_all(&save);
    }

    /// Every silo storage of the real savegame `FS25_SAVE` gets OAT added and
    /// its first fill level raised by 1000, on a copy.
    #[test]
    #[ignore = "needs FS25_SAVE pointing at a real savegame"]
    fn test_real_save_silo_edits_round_trip() {
        let src = std::path::PathBuf::from(std::env::var("FS25_SAVE").expect("FS25_SAVE"));
        let save = std::env::temp_dir().join("fs25_real_silo_edits");
        let _ = std::fs::remove_dir_all(&save);
        std::fs::create_dir_all(&save).unwrap();
        std::fs::copy(src.join("placeables.xml"), save.join("placeables.xml")).unwrap();
        let before = parse_placeables(&save).unwrap();

        let changes: Vec<PlaceableChange> = before
            .iter()
            .filter(|p| !p.silo_storages.is_empty())
            .map(|p| {
                let stocks = p.silo_storages.iter().flat_map(|s| {
                    let mut edits = vec![SiloStockChange { kind: s.kind, index: s.index, fill_type: "OAT".into(), amount: 500.0 }];
                    if let Some(first) = s.stocks.first().filter(|f| f.fill_type != "OAT") {
                        edits.push(SiloStockChange { kind: s.kind, index: s.index, fill_type: first.fill_type.clone(), amount: first.amount + 1000.0 });
                    }
                    edits
                });
                silo_change(p.index, stocks.collect())
            })
            .collect();
        assert!(!changes.is_empty());
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        assert_eq!(after.len(), before.len());
        for (b, a) in before.iter().zip(&after) {
            assert_eq!(a.production_storage, b.production_storage, "{}", b.display_name);
            for (bs, as_) in b.silo_storages.iter().zip(&a.silo_storages) {
                let level = |s: &crate::models::placeable::SiloStorage, f: &str| s.stocks.iter().find(|x| x.fill_type == f).map(|x| x.amount);
                assert_eq!(level(as_, "OAT"), Some(500.0), "{}", b.display_name);
                if let Some(first) = bs.stocks.first().filter(|f| f.fill_type != "OAT") {
                    assert!((level(as_, &first.fill_type).unwrap() - first.amount - 1000.0).abs() < 0.001);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    fn objects_change(index: usize, change: ObjectStorageChange) -> PlaceableChange {
        PlaceableChange {
            index,
            farm_id: None,
            price: None,
            complete_construction: false,
            production_storage: None,
            silo_stocks: None,
            object_storage: Some(change),
        }
    }

    fn object_storage_index(save: &std::path::Path) -> usize {
        parse_placeables(save).unwrap().iter().position(|p| !p.stored_objects.is_empty()).unwrap()
    }

    fn stored(save: &std::path::Path, index: usize) -> Vec<crate::models::placeable::StoredObject> {
        parse_placeables(save).unwrap()[index].stored_objects.clone()
    }

    #[test]
    fn test_write_object_storage_removes_objects_by_position() {
        let save = reference_copy("objects_remove");
        let index = object_storage_index(&save);
        let before = stored(&save, index);
        let change = ObjectStorageChange { remove: vec![0, 5, 100], ..Default::default() };

        write_placeable_changes(&save, &[objects_change(index, change)]).unwrap();

        let expected: Vec<_> = before
            .iter()
            .enumerate()
            .filter(|(i, _)| ![0, 5, 100].contains(i))
            .map(|(_, o)| o.clone())
            .collect();
        assert_eq!(stored(&save, index), expected);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_object_storage_copies_bales_and_pallets() {
        let save = reference_copy("objects_copy");
        let index = object_storage_index(&save);
        let before = stored(&save, index);
        let bale = before.iter().position(|o| o.class_name == "Bale").unwrap();
        let pallet = before.iter().position(|o| o.class_name == "Vehicle").unwrap();
        let change = ObjectStorageChange {
            copies: vec![ObjectCopy { index: bale, count: 3 }, ObjectCopy { index: pallet, count: 2 }],
            ..Default::default()
        };
        let configurations = |save: &std::path::Path| {
            std::fs::read_to_string(save.join("placeables.xml")).unwrap().matches("<configuration ").count()
        };
        let configurations_before = configurations(&save);

        write_placeable_changes(&save, &[objects_change(index, change)]).unwrap();

        let after = stored(&save, index);
        assert_eq!(after.len(), before.len() + 5);
        assert_eq!(after[..before.len()], before[..]);
        let added: Vec<_> = after[before.len()..].to_vec();
        assert_eq!(added, vec![before[bale].clone(), before[bale].clone(), before[bale].clone(), before[pallet].clone(), before[pallet].clone()]);
        // A pallet is copied with its <configuration> children
        assert_eq!(configurations(&save), configurations_before + 2);
        let xml = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(!xml.contains("&#13;"));
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_object_storage_copies_an_object_it_removes() {
        let save = reference_copy("objects_copy_removed");
        let index = object_storage_index(&save);
        let before = stored(&save, index);
        let change = ObjectStorageChange {
            remove: vec![0],
            copies: vec![ObjectCopy { index: 0, count: 1 }],
            ..Default::default()
        };

        write_placeable_changes(&save, &[objects_change(index, change)]).unwrap();

        let after = stored(&save, index);
        assert_eq!(after.len(), before.len());
        assert_eq!(after[..before.len() - 1], before[1..]);
        assert_eq!(after.last(), before.first());
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_object_storage_finishes_fermentation() {
        let save = std::env::temp_dir().join("fs25_test_wp_fermentation");
        let _ = std::fs::remove_dir_all(&save);
        std::fs::create_dir_all(&save).unwrap();
        // Bale.lua saves a fermenting bale's state as a <fermentation> child
        std::fs::write(
            save.join("placeables.xml"),
            r#"<placeables><placeable filename="data/placeables/mapUS/objectStorage/objectStorage.xml" farmId="1"><objectStorage>
            <object className="Bale" filename="data/objects/bales/squarebales/squarebale220/squarebale220.xml" fillLevel="4000.000000" fillType="GRASS_WINDROW" variationIndex="2" farmId="1" isMissionBale="false" wrappingState="1.000000" wrappingColor="1 1 1">
                <fermentation isFermenting="true" time="1234.000000"/>
            </object>
        </objectStorage></placeable></placeables>"#,
        )
        .unwrap();
        let change = ObjectStorageChange { finish_fermentation: true, ..Default::default() };

        write_placeable_changes(&save, &[objects_change(0, change)]).unwrap();

        let xml = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(xml.contains(r#"<fermentation isFermenting="true" time="1000000000000.000000"/>"#), "{}", xml);
        let _ = std::fs::remove_dir_all(&save);
    }

    /// Copies a bale and a pallet and removes the first object of every
    /// object storage of the real savegame `FS25_SAVE`, on a copy.
    #[test]
    #[ignore = "needs FS25_SAVE pointing at a real savegame"]
    fn test_real_save_object_storage_edits_round_trip() {
        let src = std::path::PathBuf::from(std::env::var("FS25_SAVE").expect("FS25_SAVE"));
        let save = std::env::temp_dir().join("fs25_real_object_edits");
        let _ = std::fs::remove_dir_all(&save);
        std::fs::create_dir_all(&save).unwrap();
        std::fs::copy(src.join("placeables.xml"), save.join("placeables.xml")).unwrap();
        let before = parse_placeables(&save).unwrap();

        let changes: Vec<PlaceableChange> = before
            .iter()
            .filter(|p| !p.stored_objects.is_empty())
            .map(|p| {
                let mut copies = Vec::new();
                for class in ["Bale", "Vehicle"] {
                    if let Some(i) = p.stored_objects.iter().position(|o| o.class_name == class) {
                        copies.push(ObjectCopy { index: i, count: 2 });
                    }
                }
                objects_change(p.index, ObjectStorageChange { remove: vec![0], copies, finish_fermentation: true })
            })
            .collect();
        assert!(!changes.is_empty());
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        assert_eq!(after.len(), before.len());
        for (b, a) in before.iter().zip(&after) {
            assert_eq!(a.silo_storages.len(), b.silo_storages.len());
            if b.stored_objects.is_empty() {
                assert!(a.stored_objects.is_empty());
                continue;
            }
            let copies = changes.iter().find(|c| c.index == b.index).unwrap().object_storage.as_ref().unwrap().copies.len();
            assert_eq!(a.stored_objects.len(), b.stored_objects.len() - 1 + copies * 2, "{}", b.display_name);
            assert_eq!(a.stored_objects[..b.stored_objects.len() - 1], b.stored_objects[1..]);
            println!("{}: {} -> {} objects", b.display_name, b.stored_objects.len(), a.stored_objects.len());
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_owner() {
        let save = setup_fixture("owner");
        let changes = vec![PlaceableChange {
            index: 0,
            farm_id: Some(2),
            price: None,
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let placeables = parse_placeables(&save).unwrap();
        assert_eq!(placeables[0].farm_id, 2);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_complete_construction() {
        let save = setup_fixture("complete");
        // Find which index is under construction
        let before = parse_placeables(&save).unwrap();
        let uc = before.iter().find(|p| p.is_under_construction).unwrap();
        let uc_index = uc.index;

        let changes = vec![PlaceableChange {
            index: uc_index,
            farm_id: None,
            price: None,
            complete_construction: true,
            silo_stocks: None,
            object_storage: None,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();
        let p = &after[uc_index];
        // Every <input remainingAmount> is zeroed, so nothing is left to deliver.
        assert!(!p.is_under_construction);
        let construction = p.construction.as_ref().unwrap();
        for mat in &construction.inputs {
            assert!(mat.amount_remaining <= 0.0);
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_production_stock() {
        let save = setup_fixture("production");
        let before = parse_placeables(&save).unwrap();
        let prod = before
            .iter()
            .find(|p| !p.production_storage.is_empty())
            .unwrap();
        let prod_index = prod.index;
        let fill_type = prod.production_storage[0].fill_type.clone();

        let changes = vec![PlaceableChange {
            index: prod_index,
            farm_id: None,
            price: None,
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: Some(vec![ProductionStockChange {
                fill_type: fill_type.clone(),
                amount: 5000.0,
            }]),
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();
        let p = &after[prod_index];
        let stock = p
            .production_storage
            .iter()
            .find(|s| s.fill_type == fill_type)
            .unwrap();
        assert!((stock.amount - 5000.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_leaves_silo_storage_untouched() {
        let save = setup_fixture("silo_untouched");
        // Fixture precondition (not a result to verify): the silo at index 0 has
        // <silo><storage><node fillType="WHEAT"/><node fillType="SUNFLOWER"/></storage></silo>,
        // which shares the <node> shape used by production points. Editing a production
        // point must not touch it. The SUNFLOWER node is checked after the write below.

        let changes = vec![PlaceableChange {
            index: 3,
            farm_id: None,
            price: None,
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: Some(vec![ProductionStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 42.0,
            }]),
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        // Production point updated...
        let stock = after[3]
            .production_storage
            .iter()
            .find(|s| s.fill_type == "WHEAT")
            .unwrap();
        assert!((stock.amount - 42.0).abs() < 0.01);
        // ...while the silo's own nodes kept their original levels.
        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""));
        assert!(raw.contains("fillType=\"SUNFLOWER\" fillLevel=\"4855.552734\""));
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_production_storage_change_does_not_patch_non_production_node() {
        // Exercises the `in_production_storage` gate directly. Index 0 is the silo:
        // its <node> elements sit under <silo><storage>, never inside a
        // <productionPoint>. A production_storage change aimed at the SILO'S OWN
        // index sees `current_change = Some(..)` while walking those nodes, so
        // whether they get patched depends entirely on the `if in_production_storage`
        // guard on the "node" arm — not on index scoping. (A change aimed at the
        // production point's own index, as in the test above, would never reach the
        // silo regardless of the guard, because `current_change` stays None while
        // walking the silo's subtree — that would be a toothless test.)
        let save = setup_fixture("silo_gate");

        let changes = vec![PlaceableChange {
            index: 0,
            farm_id: None,
            price: None,
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: Some(vec![ProductionStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 42.0,
            }]),
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""),
            "silo's WHEAT node was patched by a production_storage change not scoped to a productionPoint"
        );
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_targets_correct_index_past_a_self_closing_placeable() {
        // 37% of placeables in a real save are self-closing. The parser counts them;
        // if the writer does not, every edit after the first one lands on the wrong
        // building. Fixture index 1 is self-closing, so index 2 is only reachable
        // if both sides agree.
        let save = setup_fixture("self_closing_index");
        let before = parse_placeables(&save).unwrap();

        let target = 2usize;
        let target_name = before[target].display_name.clone();

        let changes = vec![PlaceableChange {
            index: target,
            farm_id: Some(7),
            price: None,
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        assert_eq!(after[target].display_name, target_name, "index drifted");
        assert_eq!(after[target].farm_id, 7, "the edit did not reach its target");
        for (i, (a, b)) in after.iter().zip(before.iter()).enumerate() {
            if i != target {
                assert_eq!(a.farm_id, b.farm_id, "placeable {} was collaterally modified", i);
            }
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_can_edit_a_self_closing_placeable() {
        // The self-closing pre-placed entries are exactly the ones a player claims
        // after buying farmland, so they must be writable.
        let save = setup_fixture("self_closing_edit");
        let changes = vec![PlaceableChange {
            index: 1,
            farm_id: Some(3),
            price: None,
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        assert_eq!(after[1].farm_id, 3);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_roundtrip() {
        let save = setup_fixture("roundtrip_p");
        let before = parse_placeables(&save).unwrap();

        // Modify first placeable's price
        let changes = vec![PlaceableChange {
            index: 0,
            farm_id: None,
            price: Some(999999.0),
            complete_construction: false,
            silo_stocks: None,
            object_storage: None,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();

        assert_eq!(after.len(), before.len());
        assert!((after[0].price - 999999.0).abs() < 0.01);

        // Other placeables should be untouched
        for i in 1..before.len() {
            assert!((before[i].price - after[i].price).abs() < 0.01);
            assert_eq!(before[i].farm_id, after[i].farm_id);
        }

        let _ = std::fs::remove_dir_all(&save);
    }
}
