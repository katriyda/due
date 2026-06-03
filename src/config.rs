use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationMethod {
    SystemNotification,
    WindowPopup,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub notification_method: NotificationMethod,
    pub default_snooze_minutes: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notification_method: NotificationMethod::Both,
            default_snooze_minutes: 15,
        }
    }
}

pub fn save_config(dir: &std::path::Path, config: &AppConfig) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    let toml_str = toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {}", e))?;
    let path = dir.join(CONFIG_FILE);
    std::fs::write(&path, toml_str).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

pub fn ensure_config_file(dir: &std::path::Path) -> Result<AppConfig, String> {
    let path = dir.join(CONFIG_FILE);
    if path.exists() {
        return load_config_from_path(&path);
    }
    let config = AppConfig::default();
    save_config(dir, &config)?;
    Ok(config)
}

fn load_config_from_path(path: &std::path::Path) -> Result<AppConfig, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_both_notification_and_15min_snooze() {
        let config = AppConfig::default();

        assert_eq!(config.notification_method, NotificationMethod::Both);
        assert_eq!(config.default_snooze_minutes, 15);
    }

    #[test]
    fn save_and_load_config_roundtrip() {
        let dir = std::env::temp_dir().join("due_test_config_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let config = AppConfig {
            notification_method: NotificationMethod::SystemNotification,
            default_snooze_minutes: 30,
        };

        super::save_config(&dir, &config).unwrap();
        let loaded = super::ensure_config_file(&dir).unwrap();

        assert_eq!(loaded.notification_method, NotificationMethod::SystemNotification);
        assert_eq!(loaded.default_snooze_minutes, 30);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_config_file_returns_default_when_file_missing() {
        let dir = std::env::temp_dir().join("due_test_config_missing");
        let _ = std::fs::remove_dir_all(&dir);

        let config = super::ensure_config_file(&dir).unwrap();

        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn ensure_config_file_creates_default_when_missing() {
        let dir = std::env::temp_dir().join("due_test_ensure_config");
        let _ = std::fs::remove_dir_all(&dir);

        let config = super::ensure_config_file(&dir).unwrap();

        assert_eq!(config, AppConfig::default());
        assert!(dir.join("config.toml").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
