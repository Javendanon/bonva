use anyhow::{Context, Result, bail, ensure};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn digest(path: &Path) -> Result<String> {
    Ok(hash(
        &fs::read(path).with_context(|| format!("Read {}", path.display()))?,
    ))
}
pub fn identity(value: &Value) -> String {
    hash(&serde_json::to_vec(value).expect("JSON value"))
}
pub fn read(path: &Path) -> Result<Value> {
    decode(&fs::read(path)?).with_context(|| format!("JSON {}", path.display()))
}
pub fn decode(bytes: &[u8]) -> Result<Value> {
    Ok(serde_json::from_slice::<Unique>(bytes)?.0)
}

// serde_json::Value normally accepts duplicate keys. Evidence and policy must not.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "JSON without duplicate keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| Unique(n.into()))
                    .ok_or_else(|| E::custom("Non-finite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut v = vec![];
                while let Some(x) = a.next_element::<Unique>()? {
                    v.push(x.0);
                }
                Ok(Unique(v.into()))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut m = Map::new();
                while let Some((k, v)) = a.next_entry::<String, Unique>()? {
                    if m.insert(k.clone(), v.0).is_some() {
                        return Err(de::Error::custom(format!("Duplicate key {k}")));
                    }
                }
                Ok(Unique(m.into()))
            }
        }
        d.deserialize_any(V)
    }
}

pub fn schema(kind: &str, value: &Value) -> Result<()> {
    let doc = read(&root().join("schemas/evaluator.json"))?;
    visit(&doc["$defs"][kind], value, &doc, kind)
}
fn visit(s: &Value, v: &Value, doc: &Value, path: &str) -> Result<()> {
    if let Some(r) = s["$ref"].as_str() {
        return visit(
            doc.pointer(r.trim_start_matches('#'))
                .context("Unknown schema reference")?,
            v,
            doc,
            path,
        );
    }
    ensure!(s.is_object(), "Unknown schema at {path}");
    if let Some(t) = s.get("type") {
        let types = if let Some(a) = t.as_array() {
            a.clone()
        } else {
            vec![t.clone()]
        };
        ensure!(
            types.iter().any(|t| match t.as_str() {
                Some("null") => v.is_null(),
                Some("boolean") => v.is_boolean(),
                Some("integer") => v.is_i64() || v.is_u64(),
                Some("number") => v.is_number(),
                Some("string") => v.is_string(),
                Some("object") => v.is_object(),
                Some("array") => v.is_array(),
                _ => false,
            }),
            "Invalid type: {path}"
        );
    }
    if let Some(c) = s.get("const") {
        ensure!(v == c, "Invalid constant: {path}");
    }
    if let Some(e) = s["enum"].as_array() {
        ensure!(e.contains(v), "Invalid enum: {path}");
    }
    if let Some(n) = v.as_f64() {
        if let Some(m) = s["minimum"].as_f64() {
            ensure!(n >= m, "Below minimum: {path}");
        }
        if let Some(m) = s["maximum"].as_f64() {
            ensure!(n <= m, "Above maximum: {path}");
        }
    }
    if let Some(m) = v.as_object() {
        if let Some(required) = s["required"].as_array() {
            for k in required {
                ensure!(
                    m.contains_key(k.as_str().context("Schema key")?),
                    "Missing field: {path}.{k}"
                );
            }
        }
        for (k, x) in m {
            if let Some(child) = s["properties"].get(k) {
                visit(child, x, doc, &format!("{path}.{k}"))?;
            } else if s["additionalProperties"] == false {
                bail!("Unknown field: {path}.{k}");
            }
        }
    }
    if let Some(a) = v.as_array() {
        ensure!(
            a.len() >= s["minItems"].as_u64().unwrap_or(0) as usize,
            "Too few items: {path}"
        );
        if let Some(item) = s.get("items") {
            for x in a {
                visit(item, x, doc, path)?;
            }
        }
    }
    Ok(())
}
