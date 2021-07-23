/*!
The components required by a plugin host to load/unload plugins.

The primary component of the plugin host's interaction is the [`PluginManager`](struct.PluginManager.html);
this type manages the lifecycle of plugins, as well as opening and closing the necessary dynamic
libraries.

# Example

As the example below shows, the plugin manager is relatively simple in it's interface. However,
as any more complex host may require loading multiple libraries, and different types of plugins,
the [`PluginManagerConfiguration`](../config/struct.PluginManagerConfiguration.html) type is a
higher-level abstraction.

```rust,no_run
use dygpi::manager::PluginManager;
use dygpi::plugin::Plugin;
use std::sync::Arc;

# const EFFECT_PLUGIN_ID: &str = "sound_effects";
# #[derive(Debug)]
# struct SoundEffectPlugin;
# impl Plugin for SoundEffectPlugin {
#     fn plugin_id(&self) -> &String {
#         unimplemented!()
#     }
#     fn on_load(&self) -> dygpi::error::Result<()> { Ok(()) }
#     fn on_unload(&self) -> dygpi::error::Result<()> { Ok(()) }
# }
# impl SoundEffectPlugin {
#     pub fn play(&self) {}
# }
let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

plugin_manager
    .load_plugins_from("libsound_one.dylib")
    .unwrap();

let plugin: Arc<SoundEffectPlugin> = plugin_manager
    .get("sound_one::sound_one::DelayEffect")
    .unwrap();

println!("{}", plugin.plugin_id());

plugin.play();
```

*/

use crate::error::{Error, ErrorKind, Result};
use crate::plugin::{
    compatibility_hash, CompatibilityFn, Plugin, PluginRegistrar, PluginRegistrationFn,
    COMPATIBILITY_FN_NAME, PLUGIN_REGISTRATION_FN_NAME,
};
use libloading::{Library, Symbol};
use search_path::SearchPath;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// The plugin manager loads and unloads plugins from a library which is dynamically opened and
/// closed as necessary.
///
#[derive(Debug)]
pub struct PluginManager<T>
where
    T: Plugin,
{
    search_path: SearchPath,
    registration_fn_name: Vec<u8>,
    plugins: RwLock<HashMap<String, LoadedPlugin<T>>>,
}

#[cfg(target_os = "macos")]
/// File name extension commonly used for a dynamic library.
pub const PLATFORM_DYLIB_EXTENSION: &str = "dylib";

#[cfg(target_os = "linux")]
/// File name extension commonly used for a dynamic library.
pub const PLATFORM_DYLIB_EXTENSION: &str = "so";

#[cfg(target_os = "windows")]
/// File name extension commonly used for a dynamic library.
pub const PLATFORM_DYLIB_EXTENSION: &str = "dll";

#[cfg(target_os = "windows")]
/// Prefix for dynamic libraries, if any.
pub const PLATFORM_DYLIB_PREFIX: &str = "";

#[cfg(not(target_os = "windows"))]
/// Prefix for dynamic libraries, if any.
pub const PLATFORM_DYLIB_PREFIX: &str = "lib";

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

///
/// Given a file name, or path with a file name, return a new path that formats the file name
/// according to common platform conventions. If the file name appears to have an extension it
/// will be overwritten by the platform extension.
///
pub fn make_platform_dylib_name(file_path: &Path) -> PathBuf {
    if let Some(file_stem) = file_path.file_stem() {
        let file_name = if !PLATFORM_DYLIB_PREFIX.is_empty() {
            let mut prefixed = OsString::from(PLATFORM_DYLIB_PREFIX);
            prefixed.push(file_stem);
            prefixed
        } else {
            file_stem.to_os_string()
        };
        let mut file_path = file_path.to_path_buf();
        file_path.set_file_name(file_name);
        let _ = file_path.set_extension(PLATFORM_DYLIB_EXTENSION);
        file_path
    } else {
        file_path.to_path_buf()
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

const UTF8_STRING_PANIC: &str = "Invalid UTF8 symbol name when converting to string";

// ------------------------------------------------------------------------------------------------

impl<T> Default for PluginManager<T>
where
    T: Plugin,
{
    fn default() -> Self {
        Self {
            search_path: Default::default(),
            registration_fn_name: PLUGIN_REGISTRATION_FN_NAME.to_vec(),
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
    ///
    /// Construct a new plugin manager and have it use the values of the string slice
    /// as a search path when loading libraries.
    ///
    pub fn new_with_search_path(search_path: SearchPath) -> Self {
        Self {
            search_path,
            registration_fn_name: PLUGIN_REGISTRATION_FN_NAME.to_vec(),
            plugins: Default::default(),
        }
    }

    ///
    /// Load all plugins from the libraries that are specified in the named environment variable.
    ///
    /// The environment variable's value is assumed to be a list of paths separated by the colon,
    /// `':'` character.
    ///
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

    ///
    /// Load all plugins from the libraries specified in the string slice, each value is a file path.
    ///
    pub fn load_plugins_from_all(&mut self, file_names: &[&str]) -> Result<()> {
        info!("PluginManager::load_all_plugins_from({:?})", file_names);
        for file_name in file_names {
            self.load_plugins_from(file_name)?;
        }
        Ok(())
    }

    ///
    /// Load all plugins from a single library with the provided file name/path.
    ///
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

    ///
    /// Override the default registration function name
    /// [`PLUGIN_REGISTRATION_FN_NAME`](../plugin/const.PLUGIN_REGISTRATION_FN_NAME.html).
    ///
    /// This function **must** conform to the type
    /// [`PluginRegistrationFn`](../plugin/function.PluginRegistrationFn.html), and must be marked
    /// as `#[no_mangle] pub extern "C"` in the same manner as the standard registration function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dygpi::plugin::{Plugin, PluginRegistrar};
    /// # #[derive(Debug)]
    /// # struct SoundSourcePlugin;
    /// # impl Plugin for SoundSourcePlugin {
    /// #     fn plugin_id(&self) -> &String {
    /// #         unimplemented!()
    /// #     }
    /// #     fn on_load(&self) -> dygpi::error::Result<()> { Ok(()) }
    /// #     fn on_unload(&self) -> dygpi::error::Result<()> { Ok(()) }
    /// # }
    /// # impl SoundSourcePlugin {
    /// #     pub fn new(id: &str) -> Self { Self {} }
    /// # }
    /// # #[derive(Debug)]
    /// # struct SoundEffectPlugin;
    /// # impl Plugin for SoundEffectPlugin {
    /// #     fn plugin_id(&self) -> &String {
    /// #         unimplemented!()
    /// #     }
    /// #     fn on_load(&self) -> dygpi::error::Result<()> { Ok(()) }
    /// #     fn on_unload(&self) -> dygpi::error::Result<()> { Ok(()) }
    /// # }
    /// # impl SoundEffectPlugin {
    /// #     pub fn new(id: &str) -> Self { Self {} }
    /// # }
    /// # const PLUGIN_NAME: &str = "RandomSource";
    /// # const OTHER_PLUGIN_NAME: &str = "DelayEffect";
    ///
    /// #[no_mangle]
    /// pub extern "C" fn register_sources(registrar: &mut PluginRegistrar<SoundSourcePlugin>) {
    ///     registrar.register(SoundSourcePlugin::new(PLUGIN_NAME));
    /// }
    ///
    /// #[no_mangle]
    /// pub extern "C" fn register_effects(registrar: &mut PluginRegistrar<SoundEffectPlugin>) {
    ///     registrar.register(SoundEffectPlugin::new(OTHER_PLUGIN_NAME));
    /// }
    /// ```
    ///
    pub fn set_registration_fn_name(&mut self, name: &[u8]) {
        self.registration_fn_name = name.to_vec()
    }

    ///
    /// Returns `true` if the plugin manager has no plugins registered, else `false`.
    ///
    pub fn is_empty(&self) -> bool {
        self.plugins.read().unwrap().is_empty()
    }

    ///
    /// Return the number of plugins registered in this plugin manager.
    ///
    pub fn len(&self) -> usize {
        self.plugins.read().unwrap().len()
    }

    ///
    /// Returns `true` if this plugin manager has a registered plugin with the provided plugin
    /// identifier, else `false`.
    pub fn contains(&self, plugin_id: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(plugin_id)
    }

    ///
    /// Returns the plugin with the provided plugin identifier, if one exists, else `None`.
    pub fn get(&self, plugin_id: &str) -> Option<Arc<T>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).map(|p| p.plugin.clone())
    }

    ///
    /// Return all the plugins registered in this plugin manager as a vector.
    ///
    pub fn plugins(&self) -> Vec<Arc<T>> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| p.plugin.clone()).collect()
    }

    ///
    /// Unload all plugins, and associated libraries, that are currently registered in this
    /// plugin manager.
    ///
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

    ///
    /// Unload the plugin identified by the provided plugin identifier, if one exists. Note that
    /// this method will also close the plugin library if no other plugins are using it.
    ///
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
        if let Some(path) = self.search_path.find_file(file_name.as_ref()) {
            return path.to_string_lossy().to_string();
        } else {
            file_name.to_string()
        }
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
                .get(self.registration_fn_name.as_slice())
                .map_err(|e| {
                    Error::from(ErrorKind::SymbolNotFound(
                        String::from_utf8(self.registration_fn_name.clone())
                            .expect(UTF8_STRING_PANIC),
                        Box::new(e),
                    ))
                })?;
            loader_fn
        };

        trace!(
            "PluginManager::register_plugins() > calling `{}`",
            String::from_utf8(self.registration_fn_name.clone()).expect(UTF8_STRING_PANIC)
        );
        let mut registrar = PluginRegistrar::default();
        load_fn(&mut registrar);

        let mut registry = self.plugins.write().unwrap();

        let from_library = Arc::new(from_library);

        for plugin in registrar
            .plugins()
            .map_err(|e| Error::from(ErrorKind::PluginRegistration(e)))?
        {
            info!("PluginManager::register_plugins() > calling plugin `on_load`");
            plugin.on_load()?;
            if let Some(_) = registry.insert(
                plugin.plugin_id().to_string(),
                LoadedPlugin {
                    plugin,
                    in_library: from_library.clone(),
                },
            ) {
                warn!("New plugin replaced a plugin with the same ID");
            }
        }

        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    const EXPECTED_FILE: &str = "libmy_lib.dylib";

    #[cfg(target_os = "linux")]
    const EXPECTED_FILE: &str = "libmy_lib.so";

    #[cfg(target_os = "windows")]
    const EXPECTED_FILE: &str = "my_lib.dll";

    #[test]
    fn test_make_dylib_name() {
        let file_name = make_platform_dylib_name("my_lib".as_ref());
        assert_eq!(file_name.to_str().unwrap(), EXPECTED_FILE);
        let file_name = make_platform_dylib_name("my_lib.foo".as_ref());
        assert_eq!(file_name.to_str().unwrap(), EXPECTED_FILE);
    }
}
