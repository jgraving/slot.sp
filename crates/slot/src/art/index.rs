use serde::de::{MapAccess, Visitor};
use serde_json::Value;
use slot_ui::house_label::Band;

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub ground: [u8; 3],
    pub band: Option<Band>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub real: Option<String>,
    pub logo: Option<String>,
    pub look: Option<Look>,
}

/// Scan the catalog one entry at a time. The full catalog contains thousands of
/// games; retaining its JSON tree and a second CRC map can exhaust device RAM.
pub fn find(json: &[u8], crc: u32) -> Result<Option<Entry>, serde_json::Error> {
    struct Match(u32);
    impl<'de> Visitor<'de> for Match {
        type Value = Option<Entry>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an artwork catalog")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut found = None;
            while let Some((key, raw)) = map.next_entry::<String, Value>()? {
                let alias = raw["crcs"].as_array().is_some_and(|list| {
                    list.iter().any(|v| v.as_str().and_then(crc_of) == Some(self.0))
                });
                if crc_of(&key) == Some(self.0) || alias {
                    found = Some(entry(&raw));
                }
            }
            Ok(found)
        }
    }
    let mut decoder = serde_json::Deserializer::from_slice(json);
    let found = serde::de::Deserializer::deserialize_map(&mut decoder, Match(crc))?;
    decoder.end()?;
    Ok(found)
}

fn crc_of(key: &str) -> Option<u32> {
    match key.len() == 8 {
        true => u32::from_str_radix(key, 16).ok(),
        false => None,
    }
}

fn entry(raw: &Value) -> Entry {
    let path = |k: &str| raw[k].as_str().map(str::to_string);
    Entry {
        real: path("support-texture"),
        logo: path("wheel"),
        look: look(&raw["look"]),
    }
}

fn rgb(v: &Value) -> Option<[u8; 3]> {
    let s = v.as_str()?;
    if s.len() != 6 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let at = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some([at(0)?, at(2)?, at(4)?])
}

fn look(raw: &Value) -> Option<Look> {
    if raw["sure"].as_bool() != Some(true) {
        return None;
    }
    let ground = rgb(&raw["ground"])?;
    let b = &raw["band"];
    if b.is_null() {
        return Some(Look { ground, band: None });
    }
    let size = b["size"].as_f64()? as f32;
    if !(size > 0.0 && size < 0.5) {
        return None;
    }
    let band = Band::parse(b["edge"].as_str()?, size, &rgb(&b["colour"])?)?;
    Some(Look {
        ground,
        band: Some(band),
    })
}

#[cfg(test)]
mod tests {
    use super::find;

    #[test]
    fn picks_only_the_current_game_and_its_aliases() {
        let json = br#"{"000206C0":{"support-texture":"GB/first.png"},"not-a-crc":{"crcs":["12345678"],"support-texture":"GBC/alias.png"},"87654321":{"wheel":"GBA/wheel.png"}}"#;
        assert_eq!(find(json, 0x000206c0).unwrap().unwrap().real.as_deref(), Some("GB/first.png"));
        assert_eq!(find(json, 0x12345678).unwrap().unwrap().real.as_deref(), Some("GBC/alias.png"));
        assert!(find(json, 0x99999999).unwrap().is_none());
        assert!(find(b"{broken", 0x000206c0).is_err());
    }
}
