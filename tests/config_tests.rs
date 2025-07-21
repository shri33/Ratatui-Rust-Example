// tests/config_tests.rs
use tui_image_viewer::app::config::Config;
use toml;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.theme_index, 0);
    assert_eq!(config.default_tab, 0);
    assert_eq!(config.auto_save, true);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let serialized = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();
    assert_eq!(config.theme_index, deserialized.theme_index);
    assert_eq!(config.default_tab, deserialized.default_tab);
    assert_eq!(config.auto_save, deserialized.auto_save);
}
