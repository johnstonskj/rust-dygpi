/*!
One-line description.

More detailed description, with

# Example

 */

use dygpi::plugin::PluginRegistrar;
use sound_api::SoundEffectPlugin;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn register_plugins(registrar: &mut PluginRegistrar<SoundEffectPlugin>) {
    registrar.register(SoundEffectPlugin::new(PLUGIN_NAME));
}

#[no_mangle]
pub extern "C" fn register_other_plugins(registrar: &mut PluginRegistrar<SoundEffectPlugin>) {
    registrar.register(SoundEffectPlugin::new(OTHER_PLUGIN_NAME));
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

const PLUGIN_NAME: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "::",
    module_path!(),
    "::",
    "DelayEffect"
);

const OTHER_PLUGIN_NAME: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "::",
    module_path!(),
    "::",
    "ReverbEffect"
);

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------
