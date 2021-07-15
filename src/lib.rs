/*!
Provides support for _Dynamic Generic PlugIns_, library based plugins for Rust.

More detailed description, with

# Example

```rust
use dygpi::manager::PluginManager;
use dygpi::plugin::Plugin;
use dygpi::MyPlugin;
use std::sync::Arc;

fn main() {
    pretty_env_logger::init();

    let mut plugin_manager: PluginManager<MyPlugin> = PluginManager::default();

    plugin_manager
        .load_plugins_from("libdygpi_ex1.dylib")
        .unwrap();

    let plugin: Arc<MyPlugin> = plugin_manager
        .get("dygpi_ex1::dygpi_ex1::MyPlugin")
        .unwrap();

    println!("{}", plugin.name());

    plugin.hello();
}
```

# Features

`config_serde`:

*/

#![warn(
    // ---------- Stylistic
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    // ---------- Public
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    // ---------- Unsafe
    unsafe_code,
    // ---------- Unused
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
)]

#[macro_use]
extern crate log;

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

pub mod config;

pub mod error;

pub mod plugin;

pub mod manager;
