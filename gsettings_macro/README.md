# GSettings Macro

[![github](https://img.shields.io/badge/github-seadve/gsettings-macro)](https://github.com/SeaDve/gsettings-macro)
[![crates.io](https://img.shields.io/crates/v/gsettings-macro)](https://crates.io/crates/gsettings-macro)
[![docs](https://docs.rs/gsettings-macro/badge.svg)](https://docs.rs/gsettings-macro/)
[![CI](https://github.com/SeaDve/gsettings-macro/actions/workflows/ci.yml/badge.svg)](https://github.com/SeaDve/gsettings-macro/actions/workflows/ci.yml)

Macro for typesafe GSettings key access

The macro's main purpose is to reduce the risk of mistyping a key,
using the wrong method to access values, inputting incorrect values,
and to reduce boilerplate. Additionally, the summary, description,
and default value are included in the documentation of each generated
method. This would be beneficial if you use tools like
[`rust-analyzer`](https://rust-analyzer.github.io/).

## Example

```rust,ignore
use gsettings_macro::gen_settings;
use gio::glib;

use std::path::{Path, PathBuf};

#[gen_settings(
    file = "./tests/io.github.seadve.test.gschema.xml",
    id = "io.github.seadve.test"
)]
#[gen_settings_define(
    key_name = "cache-dir",
    arg_type = "&Path",
    ret_type = "PathBuf"
)]
#[gen_settings_skip(signature = "(ss)")]
pub struct ApplicationSettings;

let settings = ApplicationSettings::default();

// `i` D-Bus type
settings.set_window_width(100);
assert_eq!(settings.window_width(), 100);

// enums
settings.set_alert_sound(AlertSound::Glass);
assert_eq!(settings.alert_sound(), AlertSound::Glass);

// bitflags
settings.set_space_style(SpaceStyle::BEFORE_COLON | SpaceStyle::BEFORE_COMMA);
assert_eq!(
    settings.space_style(),
    SpaceStyle::BEFORE_COLON | SpaceStyle::BEFORE_COMMA
);

// customly defined
settings.set_cache_dir(Path::new("/some_dir/"));
assert_eq!(settings.cache_dir(), PathBuf::from("/some_dir/"));
```

For more examples and detailed information see the
[documentation](https://seadve.github.io/gsettings-macro/gsettings_macro/attr.gen_settings.html).

There are also real-world examples of this library being used in [Mousai](https://github.com/SeaDve/Mousai)
and [Kooha](https://github.com/SeaDve/Kooha), a music recognizer and a screen recorder application, respectively.

## Generated methods

The procedural macro generates the following [`gio::Settings`](https://docs.rs/gio/latest/gio/struct.Settings.html) methods
for each key in the schema:

* `set` -> `set_${key}`, which panics when writing in a readonly
  key, and `try_set_${key}`, which behaves the same as the original method.
* `get` -> `${key}`
* `connect_changed` -> `connect_${key}_changed`
* `bind` -> `bind_${key}`
* `create_action` -> `create_${key}_action`
* `default_value` -> `${key}_default_value`
* `reset` -> `reset_${key}`

## Known issues

* Not updating when the gschema file is modified
  * Use hacks like `include_str!`
  * See [#73921](https://github.com/rust-lang/rust/issues/73921) or [#55904](https://github.com/rust-lang/rust/issues/55904)

## License

Copyright 2023 Dave Patrick Caberto

This software is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at [this site](http://mozilla.org/MPL/2.0/).
