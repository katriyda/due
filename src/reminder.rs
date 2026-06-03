use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const REMINDERS_FILE: &str = "reminders.toml";

pub fn data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or("Cannot determine AppData directory")?;
    Ok(base.join("due"))
}

pub fn save_reminders(dir: &Path, reminders: &[Reminder]) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    #[derive(Serialize)]
    struct SaveList<'a> {
        reminders: &'a [Reminder],
    }

    let list = SaveList { reminders };
    let toml_str = toml::to_string_pretty(&list).map_err(|e| format!("Failed to serialize: {}", e))?;
    let path = dir.join(REMINDERS_FILE);
    std::fs::write(&path, toml_str).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

pub fn load_reminders(dir: &Path) -> Result<Vec<Reminder>, String> {
    let path = dir.join(REMINDERS_FILE);
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    #[derive(Deserialize)]
    struct LoadList {
        reminders: Vec<Reminder>,
    }

    let list: LoadList = toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))?;
    Ok(list.reminders)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub title: String,
    pub content: String,
    pub enabled: bool,
    pub completed: bool,
}

impl Reminder {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
            enabled: true,
            completed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_reminder_with_required_fields() {
        let reminder = Reminder::new("买菜", "鸡蛋、牛奶");

        assert_eq!(reminder.title, "买菜");
        assert_eq!(reminder.content, "鸡蛋、牛奶");
        assert!(reminder.enabled);
        assert!(!reminder.completed);
    }

    #[test]
    fn serialize_single_reminder_to_toml() {
        let reminder = Reminder::new("买菜", "鸡蛋、牛奶");
        let toml_str = toml::to_string(&reminder).unwrap();

        assert!(toml_str.contains("买菜"));
        assert!(toml_str.contains("鸡蛋、牛奶"));
        assert!(toml_str.contains("enabled = true"));
        assert!(toml_str.contains("completed = false"));
    }

    #[test]
    fn deserialize_reminders_from_toml() {
        let toml_str = r#"
[[reminders]]
title = "买菜"
content = "鸡蛋、牛奶"
enabled = true
completed = false

[[reminders]]
title = "开会"
content = "周报准备"
enabled = true
completed = true
"#;

        #[derive(serde::Deserialize)]
        struct ReminderList {
            reminders: Vec<Reminder>,
        }

        let list: ReminderList = toml::from_str(toml_str).unwrap();
        assert_eq!(list.reminders.len(), 2);
        assert_eq!(list.reminders[0].title, "买菜");
        assert_eq!(list.reminders[1].title, "开会");
        assert!(list.reminders[0].enabled);
        assert!(list.reminders[1].completed);
    }

    #[test]
    fn data_dir_points_to_appdata_due() {
        let dir = super::data_dir().unwrap();
        let dir_str = dir.to_string_lossy();

        assert!(dir_str.contains("due"), "path should contain 'due': {}", dir_str);
        // Windows: should be under %APPDATA% or %LOCALAPPDATA%
        assert!(
            dir_str.contains("AppData"),
            "path should be under AppData: {}",
            dir_str
        );
    }

    #[test]
    fn save_and_load_reminders_roundtrip() {
        let dir = std::env::temp_dir().join("due_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let reminders = vec![
            Reminder::new("买菜", "鸡蛋、牛奶"),
            Reminder {
                title: "开会".to_string(),
                content: "周报准备".to_string(),
                enabled: true,
                completed: true,
            },
        ];

        super::save_reminders(&dir, &reminders).unwrap();
        let loaded = super::load_reminders(&dir).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "买菜");
        assert_eq!(loaded[1].title, "开会");
        assert!(loaded[1].completed);

        // cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
