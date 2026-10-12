//! Element text helpers shared by the XML readers.
//!
//! The reader never puts `&` in a text event: each entity or character
//! reference (`&amp;`, `&#233;`) arrives as its own `Event::GeneralRef` between
//! text events. Element text therefore has to be accumulated across both kinds
//! of event until the closing tag, never taken from a single text event.

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::BytesRef;

/// The text a general reference stands for. An entity XML does not predefine
/// is kept verbatim (`&name;`) rather than silently dropped.
pub fn resolve_ref(r: &BytesRef) -> String {
    if let Ok(Some(ch)) = r.resolve_char_ref() {
        return ch.to_string();
    }
    match resolve_predefined_entity(r) {
        Some(s) => s.to_string(),
        None => format!("&{};", &**r),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_ref_handles_predefined_char_and_unknown_references() {
        assert_eq!(resolve_ref(&BytesRef::new("amp")), "&");
        assert_eq!(resolve_ref(&BytesRef::new("lt")), "<");
        assert_eq!(resolve_ref(&BytesRef::new("#233")), "é");
        assert_eq!(resolve_ref(&BytesRef::new("#xE9")), "é");
        assert_eq!(resolve_ref(&BytesRef::new("custom")), "&custom;");
    }
}
