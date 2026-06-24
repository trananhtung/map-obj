# map-obj

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

[![crates.io](https://img.shields.io/crates/v/map-obj.svg)](https://crates.io/crates/map-obj)
[![docs.rs](https://docs.rs/map-obj/badge.svg)](https://docs.rs/map-obj)
[![CI](https://github.com/trananhtung/map-obj/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/map-obj/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/map-obj.svg)](#license)

**Map over the keys and values of a JSON object.**

Transform the keys and/or values of a `serde_json::Value` object with a closure, optionally
recursing into nested objects and arrays. A faithful Rust port of the widely-used
[`map-obj`](https://www.npmjs.com/package/map-obj) npm package — the building block behind
key transforms like [`camelcase-keys`](https://crates.io/crates/camelcase-keys).

- `deep` recursion (into nested objects and objects within arrays)
- Skip entries, rename keys, transform values, or stop recursion per entry
- `__proto__` keys are dropped
- Differential-tested against the reference `map-obj` implementation (60k cases)

## Install

```toml
[dependencies]
map-obj = "0.1"
serde_json = "1"
```

## Usage

```rust
use serde_json::json;
use map_obj::{map_obj, MapEntry};

// Rename keys:
let upper = map_obj(&json!({ "a": 1, "b": 2 }), false, |key, value| {
    MapEntry::keep(key.to_uppercase(), value.clone())
});
assert_eq!(upper, json!({ "A": 1, "B": 2 }));

// Recurse with `deep`, transform values, and drop some entries:
let result = map_obj(&json!({ "keep": { "n": 2 }, "drop": 9 }), true, |key, value| {
    if key == "drop" {
        MapEntry::Skip
    } else if let Some(n) = value.as_i64() {
        MapEntry::keep(key, json!(n * 10))
    } else {
        MapEntry::keep(key, value.clone())
    }
});
assert_eq!(result, json!({ "keep": { "n": 20 } }));
```

Use [`MapEntry::keep_without_recursing`] to keep an entry but not descend into its value.

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/./commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
