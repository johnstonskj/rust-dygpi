use dygpi::manager::PluginManager;
use std::sync::Arc;
use test_api::MyPlugin;

#[test]
fn test_library_not_found() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<MyPlugin> = PluginManager::default();

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

    let mut plugin_manager: PluginManager<MyPlugin> = PluginManager::default();

    let result = plugin_manager.load_plugins_from("libtest_api.dylib");
    assert!(result.is_err());
    let err_message = format!("{:?}", result.err().unwrap());
    assert!(err_message
        .starts_with(r##"Error(SymbolNotFound("register_plugins\u{0}", DlSym { desc: "dlsym(0x"##));
    assert!(err_message.ends_with(r##", register_plugins): symbol not found" }))"##));
}

#[test]
fn test_my_plugin() {
    let _ = pretty_env_logger::try_init();

    let mut plugin_manager: PluginManager<MyPlugin> = PluginManager::default();

    plugin_manager
        .load_plugins_from("libtest_plugin.dylib")
        .unwrap();

    let plugin: Arc<MyPlugin> = plugin_manager
        .get("test_plugin::test_plugin::MyPlugin")
        .unwrap();

    plugin.hello();
}
