use dygpi::manager::{PluginManager, PLATFORM_DYLIB_EXTENSION, PLATFORM_DYLIB_PREFIX};
use sound_api::SoundEffectPlugin;
use std::path::PathBuf;
use std::sync::Arc;

fn make_dylib_name(base_name: &str) -> PathBuf {
    PathBuf::from(format!(
        "{}{}.{}",
        PLATFORM_DYLIB_PREFIX, base_name, PLATFORM_DYLIB_EXTENSION
    ))
}

#[test]
fn test_library_not_found() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    let result = plugin_manager.load_plugins_from(&make_dylib_name("unknown"));
    assert!(result.is_err());
    let err_message = format!("{:?}", result.err().unwrap());
    println!("{}", err_message);
    assert!(err_message.starts_with("Error(LibraryOpenFailed"));
}

#[test]
fn test_library_with_no_plugins() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    let result = plugin_manager.load_plugins_from(&make_dylib_name("sound_api"));
    assert!(result.is_err());
    let err_message = format!("{:?}", result.err().unwrap());
    println!("err_message: {}", err_message);
    assert!(err_message.starts_with(r##"Error(SymbolNotFound("register_plugins"##));
}

#[test]
fn test_my_plugin() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    plugin_manager
        .load_plugins_from(&make_dylib_name("sound_plugin"))
        .unwrap();

    assert!(!plugin_manager.is_empty());
    assert_eq!(plugin_manager.len(), 1);

    let plugin: Arc<SoundEffectPlugin> = plugin_manager
        .get("sound_plugin::sound_plugin::DelayEffect")
        .unwrap();

    plugin.play();
}

#[test]
fn test_my_other_plugin() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();
    plugin_manager.set_registration_fn_name(b"register_other_plugins\0");

    plugin_manager
        .load_plugins_from(&make_dylib_name("sound_plugin"))
        .unwrap();

    assert!(!plugin_manager.is_empty());
    assert_eq!(plugin_manager.len(), 1);

    let plugin: Arc<SoundEffectPlugin> = plugin_manager
        .get("sound_plugin::sound_plugin::ReverbEffect")
        .unwrap();

    plugin.play();
}
