// Field polygons of a map i3d. FieldManager.lua binds each field to the
// farmland under its center, and the field id is that farmland's id.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::parsers::attrs::attr_str;

/// A field defined in a map i3d: its node name and polygon corners in world
/// XZ coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct MapField {
    pub name: String,
    pub polygon: Vec<(f32, f32)>,
}

impl MapField {
    /// Mean of the polygon corners, used to find the farmland under the field.
    pub fn center(&self) -> (f32, f32) {
        let n = self.polygon.len().max(1) as f32;
        let (sx, sz) = self
            .polygon
            .iter()
            .fold((0.0, 0.0), |(ax, az), &(x, z)| (ax + x, az + z));
        (sx / n, sz / n)
    }

    /// Whether world point (x, z) lies inside the polygon (even-odd rule).
    pub fn contains(&self, x: f32, z: f32) -> bool {
        let mut inside = false;
        let n = self.polygon.len();
        for i in 0..n {
            let (xi, zi) = self.polygon[i];
            let (xj, zj) = self.polygon[(i + n - 1) % n];
            if (zi > z) != (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi {
                inside = !inside;
            }
        }
        inside
    }
}

/// Returns the fields under the i3d's `fields` transform group, in order.
/// Each field node holds a `polygonPoints` group whose children are the
/// corners. Translations add up from the enclosing groups; field groups
/// carry no rotation on the base game maps nor on Pallegney.
pub fn parse_map_fields(i3d: &str) -> Vec<MapField> {
    let mut reader = Reader::from_str(i3d);
    let mut fields: Vec<MapField> = Vec::new();
    // World XZ offset of every open element
    let mut offsets: Vec<(f32, f32)> = Vec::new();
    let mut fields_depth: Option<usize> = None;
    let mut points_depth: Option<usize> = None;

    loop {
        let (e, is_empty) = match reader.read_event() {
            Ok(Event::Start(e)) => (e, false),
            Ok(Event::Empty(e)) => (e, true),
            Ok(Event::End(_)) => {
                if points_depth == Some(offsets.len()) {
                    points_depth = None;
                }
                if fields_depth == Some(offsets.len()) {
                    break;
                }
                offsets.pop();
                continue;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => continue,
        };

        let parent = offsets.last().copied().unwrap_or((0.0, 0.0));
        let is_group = e.name().as_ref() == "TransformGroup";
        let (x, z) = if is_group {
            translation_xz(&attr_str(&e, "translation"))
        } else {
            (0.0, 0.0)
        };
        let world = (parent.0 + x, parent.1 + z);
        // Depth of this element: the open elements before it, plus itself
        let depth = offsets.len() + 1;

        if is_group {
            let name = attr_str(&e, "name");
            match fields_depth {
                None if name == "fields" && !is_empty => fields_depth = Some(depth),
                Some(fd) if depth == fd + 1 => {
                    fields.push(MapField { name, polygon: Vec::new() });
                }
                Some(fd) if depth == fd + 2 && name == "polygonPoints" && !is_empty => {
                    points_depth = Some(depth);
                }
                Some(_) if points_depth.is_some_and(|pd| depth == pd + 1) => {
                    if let Some(field) = fields.last_mut() {
                        field.polygon.push(world);
                    }
                }
                _ => {}
            }
        }

        if !is_empty {
            offsets.push(world);
        }
    }

    fields
}

/// X and Z of an i3d "x y z" translation, 0 when missing.
fn translation_xz(translation: &str) -> (f32, f32) {
    let mut parts = translation
        .split_whitespace()
        .map(|v| v.parse::<f32>().unwrap_or(0.0));
    let x = parts.next().unwrap_or(0.0);
    let _y = parts.next();
    let z = parts.next().unwrap_or(0.0);
    (x, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape of the fields group in data/maps/mapUS/mapUS.i3d (points trimmed)
    const MAP_US: &str = r#"<i3D name="mapUS"><Scene>
      <TransformGroup name="rocks" nodeId="13279">
        <TransformGroup name="horizontalRock02" lockedgroup="true" translation="930.25 51 383.25" rotation="0 -23 0" nodeId="13280"/>
      </TransformGroup>
      <TransformGroup name="fields" nodeId="13283">
        <TransformGroup name="field1" translation="-980 109.891 -987" nodeId="13284">
          <TransformGroup name="polygonPoints" nodeId="13285">
            <TransformGroup name="point1" translation="0 0.00783539 0" nodeId="13286"/>
            <TransformGroup name="point2" translation="0 -0.49958 187.297" nodeId="13287"/>
            <TransformGroup name="point8" translation="111 -5.18672 0" nodeId="13293"/>
          </TransformGroup>
          <TransformGroup name="nameIndicator" translation="58.1631 -3.61859 109.85" nodeId="13294">
            <Note name="Note" translation="1.07373 -0.530708 5.83325" visibility="false" nodeId="13295" text="field1" color="4278190080" fixedSize="true"/>
          </TransformGroup>
        </TransformGroup>
        <TransformGroup name="field2" translation="-860 95.6323 -752.7" nodeId="13296">
          <TransformGroup name="polygonPoints" nodeId="13297">
            <TransformGroup name="point1" translation="0 -0.209694 0" nodeId="13298"/>
            <TransformGroup name="point2" translation="103.125 -11.5381 0" nodeId="13299"/>
          </TransformGroup>
          <TransformGroup name="fieldMapIndicator" translation="49.8033 -3.00433 -104.104" nodeId="13306"/>
        </TransformGroup>
      </TransformGroup>
      <TransformGroup name="after" nodeId="20"/>
    </Scene></i3D>"#;

    #[test]
    fn test_map_fields_in_world_coordinates() {
        let fields = parse_map_fields(MAP_US);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "field1");
        assert_eq!(
            fields[0].polygon,
            vec![(-980.0, -987.0), (-980.0, -987.0 + 187.297), (-980.0 + 111.0, -987.0)]
        );
        assert_eq!(fields[1].polygon, vec![(-860.0, -752.7), (-860.0 + 103.125, -752.7)]);
    }

    #[test]
    fn test_map_fields_add_the_fields_group_translation() {
        // Shape of FS25_Pallegney's maps/Pallegney.i3d: the group is offset
        // and a point may have no translation
        let i3d = r#"<i3D><Scene>
          <TransformGroup name="fields" translation="687.316 81.446 891.254" nodeId="3465">
            <TransformGroup name="field01" translation="-57.5622 -15.3754 -109.737" nodeId="3466">
              <TransformGroup name="polygonPoints" nodeId="3467">
                <TransformGroup name="point1" nodeId="3468"/>
                <TransformGroup name="point2" translation="63.0811 4.71319 45.909" nodeId="3469"/>
              </TransformGroup>
            </TransformGroup>
          </TransformGroup></Scene></i3D>"#;
        let fields = parse_map_fields(i3d);
        let (ox, oz) = (687.316 - 57.5622, 891.254 - 109.737);
        assert_eq!(fields[0].polygon, vec![(ox, oz), (ox + 63.0811, oz + 45.909)]);
    }

    #[test]
    fn test_map_field_center_is_corner_mean() {
        let field = MapField {
            name: "f".into(),
            polygon: vec![(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0)],
        };
        assert_eq!(field.center(), (2.0, 1.0));
    }

    #[test]
    fn test_map_field_contains() {
        // An L shape: the notch at the top right is outside
        let field = MapField {
            name: "f".into(),
            polygon: vec![(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (2.0, 2.0), (2.0, 4.0), (0.0, 4.0)],
        };
        assert!(field.contains(1.0, 1.0));
        assert!(field.contains(1.0, 3.0));
        assert!(field.contains(3.0, 1.0));
        assert!(!field.contains(3.0, 3.0));
        assert!(!field.contains(5.0, 1.0));
        // Left of the shape: the ray crosses two edges
        assert!(!field.contains(-1.0, 1.0));
        assert!(!field.contains(-1.0, 3.0));
    }
}
