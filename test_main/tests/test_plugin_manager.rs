use dygpi::manager::PluginManager;
use sound_api::SoundEffectPlugin;
use std::sync::Arc;

#[test]
fn test_library_not_found() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    let result = plugin_manager.load_plugins_from("lib_unknown.dylib");
    assert!(result.is_err());
    assert_eq!(
        format!("{:?}", result.err().unwrap()), 
        "Error(LibraryOpenFailed(\"lib_unknown.dylib\", DlOpen { desc: \"dlopen(lib_unknown.dylib, 5): image not found\" }))"
            .to_string()
    );
}

#[test]
fn test_library_with_no_plugins() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    let result = plugin_manager.load_plugins_from("libsound_api.dylib");
    assert!(result.is_err());
    let err_message = format!("{:?}", result.err().unwrap());
    println!("{}", err_message);
    assert!(err_message
        .starts_with(r##"Error(SymbolNotFound("register_plugins\u{0}", DlSym { desc: "dlsym(0x"##));
    assert!(err_message.ends_with(r##", register_plugins): symbol not found" }))"##));
}

#[test]
fn test_my_plugin() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<SoundEffectPlugin> = PluginManager::default();

    plugin_manager
        .load_plugins_from("libsound_plugin.dylib")
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
        .load_plugins_from("libsound_plugin.dylib")
        .unwrap();

    assert!(!plugin_manager.is_empty());
    assert_eq!(plugin_manager.len(), 1);

    let plugin: Arc<SoundEffectPlugin> = plugin_manager
        .get("sound_plugin::sound_plugin::ReverbEffect")
        .unwrap();

    plugin.play();
}
