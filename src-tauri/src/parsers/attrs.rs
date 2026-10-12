//! Attribute readers shared by the savegame XML parsers.
//!
//! Every reader is total: a missing or unparseable attribute yields the type's
//! zero value rather than an error, because savegame attributes are frequently
//! optional and a missing one is not a parse failure.

use quick_xml::events::BytesStart;

pub fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
        .unwrap_or_default()
}

pub fn attr_f64(e: &BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

pub fn attr_u8(e: &BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

pub fn attr_u32(e: &BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element() -> BytesStart<'static> {
        let mut e = BytesStart::new("sample");
        e.push_attribute(("name", "WHEAT"));
        e.push_attribute(("amount", "12.5"));
        e.push_attribute(("bad", "not-a-number"));
        e
    }

    #[test]
    fn test_attr_str_reads_and_defaults_to_empty() {
        assert_eq!(attr_str(&element(), "name"), "WHEAT");
        assert_eq!(attr_str(&element(), "missing"), "");
    }

    #[test]
    fn test_numeric_readers_default_to_zero_on_missing_or_malformed() {
        assert!((attr_f64(&element(), "amount") - 12.5).abs() < 0.001);
        assert!((attr_f64(&element(), "missing") - 0.0).abs() < 0.001);
        assert!((attr_f64(&element(), "bad") - 0.0).abs() < 0.001);
        assert_eq!(attr_u32(&element(), "missing"), 0);
        assert_eq!(attr_u8(&element(), "bad"), 0);
    }
}
