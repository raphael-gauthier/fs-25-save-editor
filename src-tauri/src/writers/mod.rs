use std::path::Path;

pub mod career;
pub mod collectible;
pub mod contract;
pub mod economy;
pub mod environment;
pub mod farm;
pub mod field;
pub mod husbandry;
pub mod mission;
pub mod placeable;
pub mod sale;
pub mod vehicle;

/// Replaces `xml_path` with `output` through a temp file. The reader skips a
/// leading UTF-8 BOM, so it never reaches `output`: it is restored when the
/// `original` content started with one.
pub(crate) fn replace_xml_file(xml_path: &Path, original: &str, output: &[u8]) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(output.len() + 3);
    if original.starts_with('\u{feff}') {
        bytes.extend_from_slice("\u{feff}".as_bytes());
    }
    bytes.extend_from_slice(output);
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &bytes)?;
    std::fs::rename(&tmp_path, xml_path)
}
