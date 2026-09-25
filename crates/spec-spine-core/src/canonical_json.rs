//! Canonical JSON serialization for deterministic, diffable artifacts.
//!
//! Object keys are sorted, the output is pretty-printed (2-space, `\n`), and a
//! trailing newline is appended. Key sorting falls out of `serde_json`'s default
//! `Map` being a `BTreeMap` (the `preserve_order` feature is deliberately NOT
//! enabled), so routing the value through `to_value` sorts every object's keys.

use serde::Serialize;
use spec_spine_types::Error;

/// Serialize `value` to canonical JSON: sorted keys, pretty, trailing newline.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, Error> {
    // to_value -> BTreeMap-backed objects (sorted keys); pretty-print preserves
    // that order. Arrays keep their element order (callers sort where needed).
    let value = serde_json::to_value(value).map_err(|e| Error::Internal(e.to_string()))?;
    let mut out =
        serde_json::to_string_pretty(&value).map_err(|e| Error::Internal(e.to_string()))?;
    out.push('\n');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Sample {
        zebra: u8,
        apple: u8,
    }

    #[test]
    fn keys_are_sorted_and_newline_terminated() {
        let out = super::to_string(&Sample { zebra: 1, apple: 2 }).unwrap();
        let apple_at = out.find("apple").unwrap();
        let zebra_at = out.find("zebra").unwrap();
        assert!(apple_at < zebra_at, "keys must be alphabetically sorted");
        assert!(
            out.ends_with("}\n"),
            "must end with a single trailing newline"
        );
    }
}
