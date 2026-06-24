//! Integration tests exercising the public API of `map-obj`.

use map_obj::{map_obj, MapEntry};
use serde_json::json;

#[test]
fn rename_and_recurse() {
    let value = json!({ "firstName": "Ada", "address": { "zipCode": "1" } });
    let result = map_obj(&value, true, |key, value| {
        MapEntry::keep(key.to_lowercase(), value.clone())
    });
    assert_eq!(result, json!({ "firstname": "Ada", "address": { "zipcode": "1" } }));
}

#[test]
fn filter_null_values() {
    let value = json!({ "a": 1, "b": null, "c": 3 });
    let result = map_obj(&value, false, |key, value| {
        if value.is_null() {
            MapEntry::Skip
        } else {
            MapEntry::keep(key, value.clone())
        }
    });
    assert_eq!(result, json!({ "a": 1, "c": 3 }));
}

#[test]
fn stop_recursion_per_key() {
    let value = json!({ "open": { "x": 1 }, "sealed": { "y": 2 } });
    let result = map_obj(&value, true, |key, value| {
        if key == "sealed" {
            MapEntry::keep_without_recursing(format!("{key}!"), value.clone())
        } else {
            MapEntry::keep(format!("{key}!"), value.clone())
        }
    });
    assert_eq!(result, json!({ "open!": { "x!": 1 }, "sealed!": { "y": 2 } }));
}

#[test]
fn objects_inside_arrays() {
    let value = json!({ "rows": [{ "id": 1 }, { "id": 2 }] });
    let result = map_obj(&value, true, |key, value| {
        MapEntry::keep(format!("_{key}"), value.clone())
    });
    assert_eq!(result, json!({ "_rows": [{ "_id": 1 }, { "_id": 2 }] }));
}
