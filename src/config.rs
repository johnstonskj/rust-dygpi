/*!
One-line description.

More detailed description, with

# Example

*/

use crate::error::{Error, ErrorKind, Result};
use crate::manager::PluginManager;
use crate::plugin::Plugin;
use std::collections::HashMap;

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[cfg_attr(feature = "config_serde", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub struct PluginConfiguration {
    plugins: HashMap<String, Vec<String>>,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Default for PluginConfiguration {
    fn default() -> Self {
        Self {
            plugins: Default::default(),
        }
    }
}

impl PluginConfiguration {
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn plugin_types(&self) -> impl Iterator<Item = &String> {
        self.plugins.keys()
    }

    pub fn plugin_libraries_for_type(
        &self,
        plugin_type: &str,
    ) -> Option<impl Iterator<Item = &String>> {
        self.plugins.get(plugin_type).map(|vs| vs.iter())
    }

    pub fn insert(&mut self, plugin_type: &str, library_list: &[&str]) -> Option<Vec<String>> {
        self.plugins.insert(
            plugin_type.to_string(),
            library_list.iter().map(|s| s.to_string()).collect(),
        )
    }

    pub fn remove(&mut self, plugin_type: &str) -> Option<Vec<String>> {
        self.plugins.remove(plugin_type)
    }

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
        let mut config = PluginConfiguration::default();
        config.insert("sound", &["beep", "boop"]);
        config.insert("light", &["bright", "mood"]);

        println!("{}", toml::to_string(&config).unwrap());
    }

    #[test]
    fn test_serialize_json() {
        let mut config = PluginConfiguration::default();
        config.insert("sound", &["beep", "boop"]);
        config.insert("light", &["bright", "mood"]);

        println!("{}", serde_json::to_string(&config).unwrap());
    }

    #[test]
    fn test_serialize_yaml() {
        let mut config = PluginConfiguration::default();
        config.insert("sound", &["beep", "boop"]);
        config.insert("light", &["bright", "mood"]);

        println!("{}", serde_yaml::to_string(&config).unwrap());
    }
}
