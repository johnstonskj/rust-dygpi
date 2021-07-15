/*!
One-line description.

More detailed description, with

# Example

```rust,ignore
use dygpi_core::manager::PluginRegistrar;

pub struct MyPlugin {}

const PLUGIN_NAME: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "::",
    module_path!(),
    "::",
    "MyPlugin"
);

#[no_mangle]
pub extern "C" fn register_plugins(registrar: &mut PluginRegistrar<MyPlugin>) {
    registrar.register(MyPlugin::new(PLUGIN_NAME));
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

pub trait Plugin: Any + Debug + Sync + Send {
    fn plugin_id(&self) -> &String;

    fn on_load(&self) -> Result<()>;

    fn on_unload(&self) -> Result<()>;
}

pub type PluginRegistrationFn<T> = fn(registrar: &mut PluginRegistrar<T>);

pub const PLUGIN_REGISTRATION_FN_NAME: &[u8] = b"register_plugins\0";

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub(crate) type CompatibilityFn = fn() -> u64;

pub(crate) const COMPATIBILITY_FN_NAME: &[u8] = b"compatibility_hash\0";

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

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------
