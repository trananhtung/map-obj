//! # map-obj — map over the keys and values of a JSON object
//!
//! Transform the keys and/or values of a [`serde_json::Value`] object with a closure,
//! optionally recursing into nested objects and arrays. A faithful Rust port of the
//! widely-used [`map-obj`](https://www.npmjs.com/package/map-obj) npm package — the building
//! block behind key transforms like
//! [`camelcase-keys`](https://crates.io/crates/camelcase-keys).
//!
//! ```
//! use serde_json::json;
//! use map_obj::{map_obj, MapEntry};
//!
//! // Append `_` to every key:
//! let result = map_obj(&json!({ "a": 1, "b": 2 }), false, |key, value| {
//!     MapEntry::keep(format!("{key}_"), value.clone())
//! });
//! assert_eq!(result, json!({ "a_": 1, "b_": 2 }));
//! ```
//!
//! Recurse into nested objects/arrays by passing `deep = true`:
//!
//! ```
//! use serde_json::json;
//! use map_obj::{map_obj, MapEntry};
//!
//! let result = map_obj(&json!({ "a": { "b": 1 } }), true, |key, value| {
//!     MapEntry::keep(key.to_uppercase(), value.clone())
//! });
//! assert_eq!(result, json!({ "A": { "B": 1 } }));
//! ```

#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/map-obj/0.1.0")]

use serde_json::{Map, Value};

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// The result of mapping a single `(key, value)` pair.
#[derive(Debug, Clone, PartialEq)]
pub enum MapEntry {
    /// Drop this entry from the result.
    Skip,
    /// Keep the entry with the given key and value. `recurse` controls whether nested
    /// objects/arrays in the value are mapped (when `deep` is set); it defaults to `true`
    /// via [`MapEntry::keep`].
    Keep {
        /// The (possibly new) key.
        key: String,
        /// The (possibly new) value.
        value: Value,
        /// Whether to recurse into `value` when `deep` is enabled.
        recurse: bool,
    },
}

impl MapEntry {
    /// Keep the entry with the given key and value, recursing into it when `deep`.
    #[must_use]
    pub fn keep(key: impl Into<String>, value: Value) -> Self {
        Self::Keep {
            key: key.into(),
            value,
            recurse: true,
        }
    }

    /// Keep the entry but do not recurse into its value, even when `deep` is set.
    #[must_use]
    pub fn keep_without_recursing(key: impl Into<String>, value: Value) -> Self {
        Self::Keep {
            key: key.into(),
            value,
            recurse: false,
        }
    }
}

/// A value `map-obj` recurses into (a JSON object or array).
fn is_container(value: &Value) -> bool {
    value.is_object() || value.is_array()
}

/// Map over the keys and values of `object`.
///
/// `mapper` is called with each `(key, value)` and returns a [`MapEntry`]. With `deep`, the
/// mapper is also applied to nested objects (and to objects inside arrays). Keys mapped to
/// `__proto__` are dropped, matching the reference.
///
/// Non-object input is returned unchanged (the JavaScript original instead throws; this is
/// the lenient Rust equivalent).
pub fn map_obj<F>(object: &Value, deep: bool, mut mapper: F) -> Value
where
    F: FnMut(&str, &Value) -> MapEntry,
{
    match object {
        Value::Object(map) => Value::Object(map_object(map, deep, &mut mapper)),
        other => other.clone(),
    }
}

fn map_object<F>(map: &Map<String, Value>, deep: bool, mapper: &mut F) -> Map<String, Value>
where
    F: FnMut(&str, &Value) -> MapEntry,
{
    let mut target = Map::new();
    for (key, value) in map {
        let MapEntry::Keep {
            key: new_key,
            value: new_value,
            recurse,
        } = mapper(key, value)
        else {
            continue;
        };

        // Drop `__proto__` keys.
        if new_key == "__proto__" {
            continue;
        }

        let new_value = if deep && recurse && is_container(&new_value) {
            map_container(&new_value, deep, mapper)
        } else {
            new_value
        };

        target.insert(new_key, new_value);
    }
    target
}

/// Map an array's items (mapping objects within), as `map-obj`'s `mapArray` does.
fn map_array<F>(array: &[Value], deep: bool, mapper: &mut F) -> Vec<Value>
where
    F: FnMut(&str, &Value) -> MapEntry,
{
    array
        .iter()
        .map(|item| {
            if is_container(item) {
                map_container(item, deep, mapper)
            } else {
                item.clone()
            }
        })
        .collect()
}

fn map_container<F>(value: &Value, deep: bool, mapper: &mut F) -> Value
where
    F: FnMut(&str, &Value) -> MapEntry,
{
    match value {
        Value::Object(map) => Value::Object(map_object(map, deep, mapper)),
        Value::Array(array) => Value::Array(map_array(array, deep, mapper)),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn append_underscore(value: &Value, deep: bool) -> Value {
        map_obj(value, deep, |key, value| {
            MapEntry::keep(format!("{key}_"), value.clone())
        })
    }

    #[test]
    fn shallow_keys() {
        assert_eq!(
            append_underscore(&json!({ "a": 1, "b": 2 }), false),
            json!({ "a_": 1, "b_": 2 })
        );
        // Shallow: nested objects untouched.
        assert_eq!(
            append_underscore(&json!({ "a": { "b": 1 } }), false),
            json!({ "a_": { "b": 1 } })
        );
    }

    #[test]
    fn deep_and_arrays() {
        assert_eq!(
            append_underscore(&json!({ "a": { "b": 1 } }), true),
            json!({ "a_": { "b_": 1 } })
        );
        assert_eq!(
            append_underscore(&json!({ "a": [{ "b": 1 }] }), true),
            json!({ "a_": [{ "b_": 1 }] })
        );
    }

    #[test]
    fn skip() {
        let result = map_obj(&json!({ "a": 1, "_x": 2 }), false, |key, value| {
            if key.starts_with('_') {
                MapEntry::Skip
            } else {
                MapEntry::keep(key, value.clone())
            }
        });
        assert_eq!(result, json!({ "a": 1 }));
    }

    #[test]
    fn should_not_recurse() {
        let result = map_obj(
            &json!({ "a": { "b": 1 }, "stop": { "c": 2 } }),
            true,
            |key, value| {
                if key == "stop" {
                    MapEntry::keep_without_recursing(key, value.clone())
                } else {
                    MapEntry::keep(key, value.clone())
                }
            },
        );
        assert_eq!(result, json!({ "a": { "b": 1 }, "stop": { "c": 2 } }));
        // "a" recursed (no rename here, but recursion visited); "stop" not recursed.
    }

    #[test]
    fn drops_proto() {
        let result = map_obj(&json!({ "evil": 1, "ok": 2 }), false, |key, value| {
            if key == "evil" {
                MapEntry::keep("__proto__", value.clone())
            } else {
                MapEntry::keep(key, value.clone())
            }
        });
        assert_eq!(result, json!({ "ok": 2 }));
    }

    #[test]
    fn transform_values() {
        let result = map_obj(&json!({ "a": 1, "b": 2 }), false, |key, value| {
            let n = value.as_i64().unwrap_or(0) * 10;
            MapEntry::keep(key, json!(n))
        });
        assert_eq!(result, json!({ "a": 10, "b": 20 }));
    }

    #[test]
    fn non_object_passthrough() {
        assert_eq!(
            map_obj(&json!(42), true, |k, v| MapEntry::keep(k, v.clone())),
            json!(42)
        );
    }
}
