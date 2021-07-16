/*!
Provides a configuration type that can be used to map a plugin type identifier to a list of
library paths. The result is the ability to create plugin manager instances from this configuration
without having to load all the plugin provider paths programmatically.

# Example

The following uses the loaded configuration file to instantiate a plugin manager for the
`SoundEffectPlugin` type, with any plugin libraries specified in the file.

```rust
use dygpi::config::PluginManagerConfiguration;
use dygpi::manager::PluginManager;
use dygpi::plugin::Plugin;
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
# fn load_config_file() -> PluginManagerConfiguration { PluginManagerConfiguration::default() }

let config = load_config_file();

let plugin_manager: PluginManager<SoundEffectPlugin> =
    if config.contains_plugin_type(EFFECT_PLUGIN_ID) {
        config.make_manager_for_type(EFFECT_PLUGIN_ID).unwrap()
    } else {
        PluginManager::default()
    };
```

# Example - Serde

Given the following simple configuration we can save it in any format supported by Serde.

```rust
use dygpi::config::PluginManagerConfiguration;

let mut config = PluginManagerConfiguration::default();
let _ = config.insert("sound_effects", &["beep", "boop"]);
let _ = config.insert("light_effects", &["bright", "mood"]);
```

In **TOML**:

```toml
[plugins]
light_effects = ["bright", "mood"]
sound_effects = ["boop", "beep"]
```

In **JSON**:

```json
{
    "plugins": {
        "sound_effects": ["beep","boop"],
        "light_effects": ["bright","mood"]
    }
}
```

In **YAML**:

```yaml
---
plugins:
  light_effects:
    - bright
    - mood
  sound_effects:
    - beep
    - boop
```

*/

use crate::error::{Error, ErrorKind, Result};
use crate::manager::PluginManager;
use crate::plugin::Plugin;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// The plugin manager configuration itself. This is logically a map from a _plugin type identifier_
/// and a list of library paths. The type identifier allows the configuration to partition the list
/// of libraries so that multiple plugin managers may be created, for different plugin types, from
/// the same configuration value or serialized file.
///
/// Note, that if the feature "config_serde" is included this type implements the Serde
/// `Deserialize` and `Serialize` traits and so may be included in configuration files.
///
/// ```rust
/// use dygpi::config::PluginManagerConfiguration;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// pub struct MyAppConfiguration {
///     pub save_path: String,
///     pub template_path: String,
///     pub plugins: Option<PluginManagerConfiguration>,
/// }
/// ```
///
#[cfg_attr(feature = "config_serde", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub struct PluginManagerConfiguration {
    plugins: HashMap<String, HashSet<String>>,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Default for PluginManagerConfiguration {
    fn default() -> Self {
        Self {
            plugins: Default::default(),
        }
    }
}

impl PluginManagerConfiguration {
    /// Returns `true` if the configuration contains no plugin types, else `false`.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Returns the number of plugin types in the configuration, also referred to as its ‘length’.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Return an iterator over the plugin type identifiers in the configuration.
    pub fn plugin_types(&self) -> impl Iterator<Item = &String> {
        self.plugins.keys()
    }

    /// Returns `true` if the configuration has values for the provided plugin type identifier,
    /// else `false`.
    pub fn contains_plugin_type(&self, plugin_type: &str) -> bool {
        self.plugins.contains_key(plugin_type)
    }

    /// Returns an iterator over all the library paths specified for the provided plugin type
    /// identifier. This method returns `None` if the configuration has no entry for the plugin type.
    pub fn plugin_libraries_for_type(
        &self,
        plugin_type: &str,
    ) -> Option<impl Iterator<Item = &String>> {
        self.plugins.get(plugin_type).map(|vs| vs.iter())
    }

    /// Insert a list of libraries for the named plugin type; if there exists an entry for this
    /// type already it will be replaced. Note that this method will panic if the library list is
    /// empty.
    pub fn insert(&mut self, plugin_type: &str, library_list: &[&str]) -> Option<HashSet<String>> {
        assert!(!library_list.is_empty());
        self.plugins.insert(
            plugin_type.to_string(),
            library_list.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Merge a list of libraries into the configuration for the plugin type. if there exists an
    /// entry for this type already the values provided will be added to the list, if not then this
    /// acts exactly as `insert`. Note that this method will panic if the library list is empty.
    pub fn merge(&mut self, plugin_type: &str, library_list: &[&str]) {
        assert!(!library_list.is_empty());
        if let Some(libraries) = self.plugins.get_mut(plugin_type) {
            libraries.extend(library_list.iter().map(|s| s.to_string()))
        } else {
            let _ = self.insert(plugin_type, library_list);
        }
    }

    /// Removes and returns the plugin libraries for the plugin type.
    pub fn remove(&mut self, plugin_type: &str) -> Option<HashSet<String>> {
        self.plugins.remove(plugin_type)
    }

    /// Construct and return a new [`PluginManager`](../manager/struct.PluginManager.html) for
    /// plugins of type `T` using the list of libraries specified for the plugin type identifier
    /// provided. Note that this method will return an error if there is no configured library
    /// list for the provided plugin type.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use dygpi::config::PluginManagerConfiguration;
    /// use dygpi::manager::PluginManager;
    /// # use dygpi::plugin::Plugin;
    /// # #[derive(Debug)] struct SoundEngine;
    /// # #[derive(Debug)] struct MediaStream;
    /// # #[derive(Debug)]
    /// # struct SoundEffectPlugin {
    /// #     id: String,
    /// # };
    /// # impl Plugin for SoundEffectPlugin {
    /// #     fn plugin_id(&self) -> &String {
    /// #         &self.id
    /// #     }
    /// #     fn on_load(&self) -> dygpi::error::Result<()> { Ok(()) }
    /// #     fn on_unload(&self) -> dygpi::error::Result<()> { Ok(()) }
    /// # }
    /// # impl SoundEffectPlugin {
    /// #     pub fn new(id: &str) -> Self { todo!() }
    /// #     pub fn play(&self) {}
    /// # }
    /// # fn read_config_file() -> String { "[plugins]\nsound_effects = [\"libsound_one.dylib\"]".to_string() }
    ///
    /// let config_as_string = read_config_file();
    /// let config: PluginManagerConfiguration = toml::from_str(&config_as_string).unwrap();
    /// let manager: PluginManager<SoundEffectPlugin> =
    ///     config.make_manager_for_type("sound_effects")
    ///         .unwrap();
    /// ```
    pub fn make_manager_for_type<T>(&self, plugin_type: &str) -> Result<PluginManager<T>>
    where
        T: Plugin,
    {
        if let Some(library_list) = self.plugins.get(plugin_type) {
            let mut manager: PluginManager<T> = PluginManager::default();
            manager.load_plugins_from_all(
                &library_list
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<Vec<&str>>(),
            )?;
            Ok(manager)
        } else {
            Err(Error::from(ErrorKind::UnknownPluginManagerType(
                plugin_type.to_string(),
            )))
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_toml() {
        let mut config = PluginManagerConfiguration::default();
        let _ = config.insert("sound", &["beep", "boop"]);
        let _ = config.insert("light", &["bright", "mood"]);

        println!("{}", toml::to_string(&config).unwrap());
    }

    #[test]
    fn test_serialize_json() {
        let mut config = PluginManagerConfiguration::default();
        let _ = config.insert("sound", &["beep", "boop"]);
        let _ = config.insert("light", &["bright", "mood"]);

        println!("{}", serde_json::to_string(&config).unwrap());
    }

    #[test]
    fn test_serialize_yaml() {
        let mut config = PluginManagerConfiguration::default();
        let _ = config.insert("sound", &["beep", "boop"]);
        let _ = config.insert("light", &["bright", "mood"]);

        println!("{}", serde_yaml::to_string(&config).unwrap());
    }
}
