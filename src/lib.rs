/*!
Provides support for _Dynamic Generic PlugIns_, library based plugins for Rust.

This crate implements a simple plugin model that allows for loading of implementations from
external dynamic libraries at runtime.

1. The plugin _host_ defines a concrete type, the plugin _type_.
   1. The plugin _type_ **MUST** implement the trait [`Plugin`](plugin/trait.Plugin.html).
   1. It **MAY** be preferable to define the plugin _type_ in a separate plugin _API_ crate
      that both the _host_ and _provider_ depend upon.
1. The plugin _provider_ (or _library_) crate **MUST** set crate-type to `"dylib"` and `"rlib"` in
   their cargo configuration.
1. The plugin _provider_ **MUST** implement a function, named `register_plugins`, which is passed a
   registrar object to register any instances of the plugin _type_.
   1. A plugin _provider_ can use an alternate name for the registration function but this must be
      provided to the plugin manager via the
      [`set_registration_fn_name`](manager/struct.PluginManager.html#method.set_registration_fn_name)
      method.
1. The plugin _host_ then uses the [`PluginManager`](manager/struct.PluginManager.html) to load libraries,
   and register plugins, that have the same type as the plugin _type_.
1. The plugin _host_ **MAY** then use plugin manager's [`get`](manager/struct.PluginManager.html#method.get)
    method to fetch a specific plugin by _id_, **OR** use
   plugin manager's [`plugins`](manager/struct.PluginManager.html#method.plugins) method to iterate
   over all plugins.

Overriding the plugin registration function allows a plugin _host_ to provide plugins of different
types by using separate registration functions for each type.

# Example

The example below shows the plugin manager loading any plugins from a specific library and then
retrieving a single plugin by ID from the loaded set.

```rust,no_run
use dygpi::manager::PluginManager;
use dygpi::plugin::Plugin;
use std::sync::Arc;
# const EFFECT_PLUGIN_ID: &str = "sound_effects";
# #[derive(Debug)]
# struct SoundEffectPlugin;
# impl Plugin for SoundEffectPlugin {
#     fn plugin_id(&self) -> &String {
#         todo!()
#     }
#     fn on_load(&self) -> dygpi::error::Result<()> { Ok(()) }
#     fn on_unload(&self) -> dygpi::error::Result<()> { Ok(()) }
# }
# impl SoundEffectPlugin {
#     pub fn play(&self) {}
# }

fn main() {
    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    plugin_manager
        .load_plugins_from("libsound_one.dylib".as_ref())
        .unwrap();

    let plugin: Arc<SoundEffectPlugin> = plugin_manager
        .get("sound_one::sound_one::DelayEffect")
        .unwrap();

    println!("{}", plugin.plugin_id());

    plugin.play();
}
```

# Features

`config_serde`: Adds [Serde](https://serde.rs/)'s `Serialize` and `Deserialize` traits to the
[`PluginManagerConfiguration`](config/struct.PluginManagerConfiguration.html) type so that it can
be used in configuration files.

```toml
[plugins]
source = ["analog_oscillator", "lfo"]
effect = ["delay", "reverb"]
```

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
