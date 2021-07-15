/*!
One-line description.

More detailed description, with

# Example

*/

use crate::error::{Error, ErrorKind, Result};
use crate::plugin::{
    compatibility_hash, CompatibilityFn, Plugin, PluginRegistrationFn, COMPATIBILITY_FN_NAME,
    PLUGIN_REGISTRATION_FN_NAME,
};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct PluginManager<T>
where
    T: Plugin,
{
    search_path: Vec<String>,
    plugins: RwLock<HashMap<String, LoadedPlugin<T>>>,
}

#[derive(Debug)]
pub struct PluginRegistrar<T>
where
    T: Plugin,
{
    plugins: Vec<Arc<T>>,
    error: Option<Box<dyn std::error::Error>>,
}

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct LoadedPlugin<T>
where
    T: Plugin,
{
    plugin: Arc<T>,
    in_library: Arc<LoadedLibrary>,
}

#[derive(Debug)]
struct LoadedLibrary {
    file_name: String,
    library: Library,
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

const UTF8_STRING_PANIC: &str = "Invalid UTF8 symbol name when converting to string";

// ------------------------------------------------------------------------------------------------

impl<T> PluginRegistrar<T>
where
    T: Plugin,
{
    pub(crate) fn default() -> Self {
        Self {
            plugins: Default::default(),
            error: None,
        }
    }

    pub fn register(&mut self, plugin: T) {
        if self.error.is_none() {
            self.plugins.push(Arc::new(plugin));
        }
    }

    pub fn error(&mut self, error: Box<dyn std::error::Error>) {
        self.error = Some(error);
    }

    pub(crate) fn plugins(self) -> std::result::Result<Vec<Arc<T>>, Box<dyn std::error::Error>> {
        match self.error {
            None => Ok(self.plugins),
            Some(error) => Err(error),
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl<T> Default for PluginManager<T>
where
    T: Plugin,
{
    fn default() -> Self {
        Self {
            search_path: Default::default(),
            plugins: Default::default(),
        }
    }
}

impl<T> Drop for PluginManager<T>
where
    T: Plugin,
{
    fn drop(&mut self) {
        info!("PluginManager::drop()");
        self.unload_all().unwrap();
    }
}

impl<T> PluginManager<T>
where
    T: Plugin,
{
    pub fn new_with_search_env_var(env_var: &str) -> Self {
        match env::var(env_var) {
            Ok(value) => {
                let search_path: Vec<&str> = value.split(":").collect();
                Self::new_with_search_path(&search_path)
            }
            Err(e) => {
                warn!(
                    "Error retrieving environment variable '{}'; error: {}",
                    env_var, e
                );
                Default::default()
            }
        }
    }

    pub fn new_with_search_path(search_path: &[&str]) -> Self {
        Self {
            search_path: search_path.iter().map(|s| s.to_string()).collect(),
            plugins: Default::default(),
        }
    }

    pub fn load_all_plugins_from_env(&mut self, env_var: &str) -> Result<()> {
        info!("PluginManager::load_all_plugins_from_env({:?})", env_var);
        if let Ok(env_value) = env::var(env_var) {
            for file_name in env_value.split(":") {
                self.load_plugins_from(file_name)?;
            }
        } else {
            warn!("Failed to find environment variable '{}'", env_var);
        }
        Ok(())
    }

    pub fn load_plugins_from_all(&mut self, file_names: &[&str]) -> Result<()> {
        info!("PluginManager::load_all_plugins_from({:?})", file_names);
        for file_name in file_names {
            self.load_plugins_from(file_name)?;
        }
        Ok(())
    }

    #[allow(unsafe_code)]
    pub fn load_plugins_from(&mut self, file_name: &str) -> Result<()> {
        info!("PluginManager::load_plugins_from({:?})", file_name);

        let file_name = if !file_name.contains(&['/', '.'][..]) && !self.search_path.is_empty() {
            self.find_library(file_name)
        } else {
            file_name.to_string()
        };

        trace!("PluginManager::load_plugins_from() > opening library");
        let library = unsafe {
            Library::new(&file_name).map_err(|e| {
                Error::from(ErrorKind::LibraryOpenFailed(file_name.clone(), Box::new(e)))
            })?
        };

        let loaded_library = LoadedLibrary { file_name, library };

        trace!("PluginManager::load_plugins_from() > checking compatibility");
        self.check_compatibility(&loaded_library)?;

        trace!("PluginManager::load_plugins_from() > registering the plugins");
        self.register_plugins(loaded_library)?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.read().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.plugins.read().unwrap().len()
    }

    pub fn contains(&self, plugin_name: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(plugin_name)
    }

    pub fn get(&self, plugin_name: &str) -> Option<Arc<T>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_name).map(|p| p.plugin.clone())
    }

    pub fn plugins(&self) -> Vec<Arc<T>> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| p.plugin.clone()).collect()
    }

    pub fn unload_all(&mut self) -> Result<()> {
        info!("PluginManager::unload_all()");
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.write().unwrap();
            plugins.iter().map(|(n, _)| n).cloned().collect()
        };
        for name in plugin_names {
            self.unload_plugin(&name)?;
        }
        Ok(())
    }

    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<()> {
        info!("PluginManager::unload_plugin({:?})", plugin_name);
        let mut plugins = self.plugins.write().unwrap();
        if let Some(plugin) = plugins.remove(plugin_name) {
            trace!("PluginManager::unload_plugin() > calling plugin `on_unload`");
            plugin.plugin.on_unload()?;
            if Arc::strong_count(&plugin.in_library) == 1 {
                trace!("PluginManager::unload_plugin() > closing library");
                let in_library = Arc::try_unwrap(plugin.in_library).unwrap();
                if let Err(e) = in_library.library.close() {
                    error!(
                        "Error closing library {:?}; {}",
                        in_library.file_name.to_string(),
                        e
                    );
                    return Err(ErrorKind::LibraryCloseFailed(
                        in_library.file_name.to_string(),
                        Box::new(e),
                    )
                    .into());
                }
            }
        }
        Ok(())
    }

    // --------------------------------------------------------------------------------------------

    fn find_library(&self, file_name: &str) -> String {
        trace!("PluginManager::find_library() > checking search path for library");
        for path in &self.search_path {
            let mut path = PathBuf::from(path);
            path.push(file_name);
            trace!(
                "PluginManager::find_library() > {:?} is_file {}",
                path,
                path.is_file()
            );
            if path.is_file() {
                return path.to_string_lossy().to_string();
            }
        }
        file_name.to_string()
    }

    #[allow(unsafe_code)]
    fn check_compatibility(&self, library: &LoadedLibrary) -> Result<()> {
        let compatibility_fn = unsafe {
            let loader_fn: Symbol<'_, CompatibilityFn> =
                library.library.get(COMPATIBILITY_FN_NAME).map_err(|e| {
                    Error::from(ErrorKind::SymbolNotFound(
                        String::from_utf8(COMPATIBILITY_FN_NAME.to_vec()).expect(UTF8_STRING_PANIC),
                        Box::new(e),
                    ))
                })?;
            loader_fn
        };
        trace!("PluginManager::check_compatibility() > fetching library compatibility hash");
        let lib_compatibility_hash: u64 = compatibility_fn();
        trace!("PluginManager::check_compatibility() > fetching local compatibility hash");
        let local_compatibility_hash: u64 = compatibility_hash();
        if lib_compatibility_hash != local_compatibility_hash {
            error!(
                "Version incompatibility {:?} != {:?}",
                lib_compatibility_hash, local_compatibility_hash
            );
            return Err(ErrorKind::IncompatibleLibraryVersion(library.file_name.clone()).into());
        }
        trace!("PluginManager::check_compatibility() > compatibility version check passed");
        Ok(())
    }

    #[allow(unsafe_code)]
    fn register_plugins(&mut self, from_library: LoadedLibrary) -> Result<()> {
        trace!(
            "PluginManager::register_plugins(_, {:?})",
            &from_library.file_name
        );
        let load_fn = unsafe {
            let loader_fn: Symbol<'_, PluginRegistrationFn<T>> = from_library
                .library
                .get(PLUGIN_REGISTRATION_FN_NAME)
                .map_err(|e| {
                    Error::from(ErrorKind::SymbolNotFound(
                        String::from_utf8(PLUGIN_REGISTRATION_FN_NAME.to_vec())
                            .expect(UTF8_STRING_PANIC),
                        Box::new(e),
                    ))
                })?;
            loader_fn
        };

        trace!(
            "PluginManager::register_plugins() > calling `{}`",
            String::from_utf8(PLUGIN_REGISTRATION_FN_NAME.to_vec()).expect(UTF8_STRING_PANIC)
        );
        let mut registrar = PluginRegistrar::default();
        load_fn(&mut registrar);

        let mut registry = self.plugins.write().unwrap();

        let from_library = Arc::new(from_library);

        for plugin in registrar
            .plugins()
            .map_err(|e| Error::from(ErrorKind::PluginRegistration(e)))?
        {
            trace!(
                "PluginManager::register_plugins() > registering plugin: {:?}",
                plugin.plugin_id(),
            );
            info!("PluginManager::register_plugins() > calling plugin `on_load`");
            plugin.on_load()?;
            let previous = registry.insert(
                plugin.plugin_id().to_string(),
                LoadedPlugin {
                    plugin,
                    in_library: from_library.clone(),
                },
            );
            if previous.is_some() {
                warn!("New plugin replaced a plugin with the same ID");
            }
        }

        Ok(())
    }
}
