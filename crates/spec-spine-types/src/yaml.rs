//! The one YAML reader (spec 196). Every YAML the engine reads (spec
//! frontmatter, compaction plans, the delta classifier, the dependency-only
//! waiver, workflow manifests) is parsed here, by `serde-saphyr`, a parser
//! written in safe Rust, configured to read as the deprecated `serde_yaml` 0.9
//! did wherever a corpus can tell the difference:
//!
//! - booleans are only `true`/`false` (YAML 1.2), so `yes`, `no`, `on` and
//!   `off` stay strings, as a workflow's `on:` key needs;
//! - a merge key `<<` is an ordinary key, never expanded;
//! - a duplicate mapping key, a second document, an unknown tag (`!custom`)
//!   and a non-finite float are refused;
//! - anchors and aliases are expanded, under the parser's alias limits.
//!
//! The cases where the two parsers still differ are recorded in spec 196's
//! decisions and pinned by `tests/yaml_parity.rs`.

use serde::de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor};

/// A parsed YAML value. A mapping keeps its authored order; two mappings are
/// equal when they hold the same entries in any order, as `serde_yaml`'s were.
#[derive(Clone, Debug)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    Sequence(Vec<YamlValue>),
    Mapping(Vec<(YamlValue, YamlValue)>),
}

impl PartialEq for YamlValue {
    fn eq(&self, other: &Self) -> bool {
        use YamlValue::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            // YAML has one NaN, equal to itself, as `serde_yaml`'s was.
            (Float(a), Float(b)) => a == b || (a.is_nan() && b.is_nan()),
            (String(a), String(b)) => a == b,
            (Sequence(a), Sequence(b)) => a == b,
            (Mapping(a), Mapping(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(k, v)| {
                        b.iter()
                            .find(|(bk, _)| bk == k)
                            .is_some_and(|(_, bv)| bv == v)
                    })
            }
            // An integer is one value whichever width the parser chose.
            (a, b) => match (a.as_i128(), b.as_i128()) {
                (Some(x), Some(y)) => x == y,
                _ => false,
            },
        }
    }
}

impl YamlValue {
    fn as_i128(&self) -> Option<i128> {
        match self {
            Self::Int(i) => Some(i128::from(*i)),
            Self::UInt(u) => Some(i128::from(*u)),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&Vec<YamlValue>> {
        match self {
            Self::Sequence(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_mapping(&self) -> Option<&Vec<(YamlValue, YamlValue)>> {
        match self {
            Self::Mapping(m) => Some(m),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// The value under the string key `key`, when this is a mapping.
    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        self.as_mapping()?
            .iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .map(|(_, v)| v)
    }

    /// The value as JSON, for typed deserialization and for transport. A
    /// mapping key must be a string; anything else is refused, because a JSON
    /// object cannot carry it.
    pub fn to_json(&self) -> Result<serde_json::Value, String> {
        Ok(match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(b) => serde_json::Value::Bool(*b),
            Self::Int(i) => serde_json::Value::from(*i),
            Self::UInt(u) => serde_json::Value::from(*u),
            Self::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| format!("non-finite number {f} is not JSON-representable"))?,
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Sequence(items) => {
                serde_json::Value::Array(items.iter().map(Self::to_json).collect::<Result<_, _>>()?)
            }
            Self::Mapping(entries) => {
                let mut out = serde_json::Map::new();
                for (k, v) in entries {
                    let Some(key) = k.as_str() else {
                        return Err("mapping keys must be strings".into());
                    };
                    out.insert(key.to_string(), v.to_json()?);
                }
                serde_json::Value::Object(out)
            }
        })
    }
}

impl<'de> de::Deserialize<'de> for YamlValue {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = YamlValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a YAML value")
            }
            fn visit_unit<E>(self) -> Result<YamlValue, E> {
                Ok(YamlValue::Null)
            }
            fn visit_none<E>(self) -> Result<YamlValue, E> {
                Ok(YamlValue::Null)
            }
            fn visit_some<D2: de::Deserializer<'de>>(self, d: D2) -> Result<YamlValue, D2::Error> {
                <YamlValue as de::Deserialize>::deserialize(d)
            }
            fn visit_bool<E>(self, b: bool) -> Result<YamlValue, E> {
                Ok(YamlValue::Bool(b))
            }
            fn visit_i64<E>(self, i: i64) -> Result<YamlValue, E> {
                Ok(YamlValue::Int(i))
            }
            fn visit_u64<E>(self, u: u64) -> Result<YamlValue, E> {
                Ok(YamlValue::UInt(u))
            }
            fn visit_f64<E>(self, f: f64) -> Result<YamlValue, E> {
                Ok(YamlValue::Float(f))
            }
            fn visit_str<E>(self, s: &str) -> Result<YamlValue, E> {
                Ok(YamlValue::String(s.to_string()))
            }
            fn visit_string<E>(self, s: String) -> Result<YamlValue, E> {
                Ok(YamlValue::String(s))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<YamlValue, A::Error> {
                let mut out = Vec::new();
                while let Some(item) = a.next_element()? {
                    out.push(item);
                }
                Ok(YamlValue::Sequence(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<YamlValue, A::Error> {
                let mut out = Vec::new();
                while let Some(entry) = a.next_entry()? {
                    out.push(entry);
                }
                Ok(YamlValue::Mapping(out))
            }
        }
        d.deserialize_any(V)
    }
}

fn options() -> serde_saphyr::Options {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.merge_keys = serde_saphyr::MergeKeyPolicy::AsOrdinary;
    options.reject_unsupported_tags = true;
    options.reject_non_finite_typeless_float = true;
    options
}

/// Parse one YAML document. An empty document is `Null`.
pub fn parse(src: &str) -> Result<YamlValue, String> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    if src.trim().is_empty() {
        return Ok(YamlValue::Null);
    }
    serde_saphyr::from_str_with_options::<YamlValue>(src, options()).map_err(|e| e.to_string())
}

/// Parse one YAML document into `T`. The value is read as `serde_yaml`'s
/// `Value` deserializer read it: a null is an empty sequence or mapping where
/// one is expected, and an absent option otherwise (spec 196 D-2).
pub fn from_str<T: DeserializeOwned>(src: &str) -> Result<T, String> {
    from_value(&parse(src)?)
}

/// Deserialize `T` from a parsed value, under [`from_str`]'s rules.
pub fn from_value<T: DeserializeOwned>(value: &YamlValue) -> Result<T, String> {
    T::deserialize(value).map_err(|e| e.0)
}

/// A typed read's refusal, as text.
#[derive(Debug)]
pub struct YamlError(String);

impl std::fmt::Display for YamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for YamlError {}

impl de::Error for YamlError {
    fn custom<M: std::fmt::Display>(msg: M) -> Self {
        Self(msg.to_string())
    }
}

struct Seq<'a>(std::slice::Iter<'a, YamlValue>);

impl<'de> SeqAccess<'de> for Seq<'de> {
    type Error = YamlError;
    fn next_element_seed<S: de::DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, YamlError> {
        self.0.next().map(|v| seed.deserialize(v)).transpose()
    }
}

struct Map<'a> {
    entries: std::slice::Iter<'a, (YamlValue, YamlValue)>,
    value: Option<&'a YamlValue>,
}

impl<'de> MapAccess<'de> for Map<'de> {
    type Error = YamlError;
    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, YamlError> {
        match self.entries.next() {
            Some((k, v)) => {
                self.value = Some(v);
                seed.deserialize(k).map(Some)
            }
            None => Ok(None),
        }
    }
    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, YamlError> {
        let value = self
            .value
            .take()
            .ok_or_else(|| YamlError("a mapping value was read before its key".into()))?;
        seed.deserialize(value)
    }
}

static EMPTY_SEQ: Vec<YamlValue> = Vec::new();
static EMPTY_MAP: Vec<(YamlValue, YamlValue)> = Vec::new();

impl<'de> de::Deserializer<'de> for &'de YamlValue {
    type Error = YamlError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, YamlError> {
        match self {
            YamlValue::Null => visitor.visit_unit(),
            YamlValue::Bool(b) => visitor.visit_bool(*b),
            YamlValue::Int(i) => visitor.visit_i64(*i),
            YamlValue::UInt(u) => visitor.visit_u64(*u),
            YamlValue::Float(f) => visitor.visit_f64(*f),
            YamlValue::String(s) => visitor.visit_borrowed_str(s),
            YamlValue::Sequence(items) => visitor.visit_seq(Seq(items.iter())),
            YamlValue::Mapping(entries) => visitor.visit_map(Map {
                entries: entries.iter(),
                value: None,
            }),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, YamlError> {
        match self {
            YamlValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, YamlError> {
        match self {
            YamlValue::Null => visitor.visit_seq(Seq(EMPTY_SEQ.iter())),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, YamlError> {
        match self {
            YamlValue::Null => visitor.visit_map(Map {
                entries: EMPTY_MAP.iter(),
                value: None,
            }),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, YamlError> {
        self.deserialize_map(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, YamlError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, YamlError> {
        match self {
            YamlValue::String(s) => visitor.visit_enum(
                de::IntoDeserializer::<YamlError>::into_deserializer(s.as_str()),
            ),
            YamlValue::Mapping(entries) if entries.len() == 1 => {
                visitor.visit_enum(de::value::MapAccessDeserializer::new(Map {
                    entries: entries.iter(),
                    value: None,
                }))
            }
            _ => Err(de::Error::custom(
                "an enum is a string or a mapping with one entry",
            )),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, YamlError> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct tuple tuple_struct identifier
    }
}

#[cfg(test)]
mod tests {
    use super::YamlValue;

    #[test]
    fn nan_equals_itself_as_serde_yaml_did() {
        assert_eq!(YamlValue::Float(f64::NAN), YamlValue::Float(f64::NAN));
        let m = |v| YamlValue::Mapping(vec![(YamlValue::String("k".into()), v)]);
        assert_eq!(m(YamlValue::Float(f64::NAN)), m(YamlValue::Float(f64::NAN)));
        assert_ne!(YamlValue::Float(f64::NAN), YamlValue::Float(0.0));
    }
}
