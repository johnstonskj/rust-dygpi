/*!
The components required to define a plugin API.

The types defined in this module are required in defining the plugin API, the

# Example - Define Plugin

```rust
use dygpi::plugin::Plugin;

# #[derive(Debug)] struct SoundEngine;
# #[derive(Debug)] struct MediaStream;
#[derive(Debug)]
struct SoundEffectPlugin {
    id: String,
    engine: SoundEngine,
    media: MediaStream,
};

impl Plugin for SoundEffectPlugin {
    fn plugin_id(&self) -> &String {
        &self.id
    }

    fn on_load(&self) -> dygpi::error::Result<()> {
        // connect to sound engine
        // load media stream
        Ok(())
    }

    fn on_unload(&self) -> dygpi::error::Result<()> {
        // unload media stream
        // disconnect from sound engine
        Ok(())
    }
}

impl SoundEffectPlugin {
    pub fn new(id: &str) -> Self { todo!() }
    pub fn play(&self) {}
}
```

# Example - Register Plugin

```rust
use dygpi::manager::PluginRegistrar;
# use dygpi::plugin::Plugin;
# #[derive(Debug)] struct SoundEngine;
# #[derive(Debug)] struct MediaStream;
# #[derive(Debug)]
# struct SoundEffectPlugin {
#     id: String,
#     engine: SoundEngine,
#     media: MediaStream,
# };
# impl Plugin for SoundEffectPlugin {
#     fn plugin_id(&self) -> &String {
#         &self.id
#     }
#     fn on_load(&self) -> dygpi::error::Result<()> { Ok(()) }
#     fn on_unload(&self) -> dygpi::error::Result<()> { Ok(()) }
# }
# impl SoundEffectPlugin {
#     pub fn new(id: &str) -> Self { todo!() }
#     pub fn play(&self) {}
# }

const PLUGIN_ID: &str = concat!(env!("CARGO_PKG_NAME"), "::", module_path!(), "::DelayEffect");

#[no_mangle]
pub extern "C" fn register_plugins<MyPlugin>(
    registrar: &mut PluginRegistrar<SoundEffectPlugin>
) {
    registrar.register(SoundEffectPlugin::new(PLUGIN_ID));
}
```

*/

use crate::error::Result;
use crate::manager::PluginRegistrar;
use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// This trait must be implemented by any plugin type, it not only provides a plugin id, but also
/// provides lifecycle methods which implementors can use to manage resources owned by the plugin.
pub trait Plugin: Any + Debug + Sync + Send {
    ///
    /// Return the plug-in identifier for this instance. In general a unique format that also
    /// provides some debug/trace value is to use the package/module path as shown below.
    ///
    /// ```rust
    /// const PLUGIN_ID: &str = concat!(env!("CARGO_PKG_NAME"), "::", module_path!(), "::MyPlugin");
    /// ```
    fn plugin_id(&self) -> &String;

    ///
    /// Called by the plugin manager after the registration process is complete.
    ///
    fn on_load(&self) -> Result<()>;

    ///
    /// Called by the plugin manager once a plugin has been de-registered but before the library
    /// is closed.
    ///
    fn on_unload(&self) -> Result<()>;
}

///
/// The type for the registration function that a plugin provider **MUST** include in their
/// library. This function constructs plugin instances and uses the registrar as a callback
/// into the plugin manager.
///
/// ```rust
/// use dygpi::manager::PluginRegistrar;
/// # use dygpi::plugin::Plugin;
///
/// # #[derive(Debug)] struct SoundEngine;
/// # #[derive(Debug)] struct MediaStream;
/// # #[derive(Debug)]
/// # struct SoundEffectPlugin {
/// #     id: String,
/// #     engine: SoundEngine,
/// #     media: MediaStream,
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
/// # const PLUGIN_ID: &str = concat!(env!("CARGO_PKG_NAME"), "::", module_path!(), "::DelayEffect");
/// #[no_mangle]
/// pub extern "C" fn register_plugins<MyPlugin>(registrar: &mut PluginRegistrar<SoundEffectPlugin>) {
///     registrar.register(SoundEffectPlugin::new(PLUGIN_ID));
/// }
/// ```
///
pub type PluginRegistrationFn<T> = fn(registrar: &mut PluginRegistrar<T>);

///
/// The required name of the registration function (see the
/// [`PluginRegistrationFn`](type.PluginRegistrationFn.html) type).
///
pub const PLUGIN_REGISTRATION_FN_NAME: &[u8] = b"register_plugins\0";

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub(crate) type CompatibilityFn = fn() -> u64;

pub(crate) const COMPATIBILITY_FN_NAME: &[u8] = b"compatibility_hash\0";

///
/// This function is exposed so that the version linked into a plugin provider may be compared to
/// the one linked into the plugin host.
///
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn compatibility_hash() -> u64 {
    const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    const RUSTC_VERSION: &str = env!("RUSTC_VERSION");

    debug!(
        "compatibility_hash() -> Hash({:?}, {:?})",
        CARGO_PKG_VERSION, RUSTC_VERSION
    );

    let mut s = DefaultHasher::new();
    CARGO_PKG_VERSION.hash(&mut s);
    RUSTC_VERSION.hash(&mut s);
    s.finish()
}
