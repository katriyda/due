use crate::reminder::Reminder;

/// UI 友好的提醒数据
#[derive(Debug, Clone, PartialEq)]
pub struct ReminderItem {
    pub id: usize,
    pub title: String,
    pub content: String,
    pub enabled: bool,
    pub completed: bool,
    pub next_trigger: Option<String>,
}

/// 转换 Reminder 列表到 UI 数据
pub fn to_reminder_items(reminders: &[Reminder]) -> Vec<ReminderItem> {
    reminders
        .iter()
        .enumerate()
        .map(|(i, r)| ReminderItem {
            id: i,
            title: r.title.clone(),
            content: r.content.clone(),
            enabled: r.enabled,
            completed: r.completed,
            next_trigger: None, // 后续集成 time 模块计算
        })
        .collect()
}

/// 添加新提醒
pub fn add_reminder(reminders: &mut Vec<Reminder>, title: &str, content: &str) {
    reminders.push(Reminder::new(title, content));
}

/// 删除提醒
pub fn delete_reminder(reminders: &mut Vec<Reminder>, index: usize) -> Result<(), String> {
    if index >= reminders.len() {
        return Err(format!("索引越界: {} (共 {} 条)", index, reminders.len()));
    }
    reminders.remove(index);
    Ok(())
}

/// 切换启用状态
pub fn toggle_enabled(reminders: &mut Vec<Reminder>, index: usize) -> Result<(), String> {
    if index >= reminders.len() {
        return Err(format!("索引越界: {} (共 {} 条)", index, reminders.len()));
    }
    reminders[index].enabled = !reminders[index].enabled;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_reminder_items_converts_list() {
        let reminders = vec![
            Reminder::new("喝水", "每小时喝一杯水"),
            Reminder::new("站立", "久坐后站起来活动"),
        ];

        let items = to_reminder_items(&reminders);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "喝水");
        assert_eq!(items[1].title, "站立");
        assert_eq!(items[0].id, 0);
        assert_eq!(items[1].id, 1);
    }

    #[test]
    fn add_reminder_appends_to_list() {
        let mut reminders = vec![Reminder::new("喝水", "每小时喝一杯水")];

        add_reminder(&mut reminders, "站立", "久坐后站起来活动");

        assert_eq!(reminders.len(), 2);
        assert_eq!(reminders[1].title, "站立");
    }

    #[test]
    fn delete_reminder_removes_item() {
        let mut reminders = vec![
            Reminder::new("喝水", "每小时喝一杯水"),
            Reminder::new("站立", "久坐后站起来活动"),
        ];

        delete_reminder(&mut reminders, 0).unwrap();

        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].title, "站立");
    }

    #[test]
    fn delete_reminder_out_of_bounds_returns_error() {
        let mut reminders = vec![Reminder::new("喝水", "每小时喝一杯水")];

        let result = delete_reminder(&mut reminders, 5);

        assert!(result.is_err());
        assert_eq!(reminders.len(), 1);
    }

    #[test]
    fn toggle_enabled_flips_state() {
        let mut reminders = vec![Reminder::new("喝水", "每小时喝一杯水")];

        toggle_enabled(&mut reminders, 0).unwrap();
        assert!(!reminders[0].enabled);

        toggle_enabled(&mut reminders, 0).unwrap();
        assert!(reminders[0].enabled);
    }

    #[test]
    fn toggle_enabled_out_of_bounds_returns_error() {
        let mut reminders = vec![Reminder::new("喝水", "每小时喝一杯水")];

        let result = toggle_enabled(&mut reminders, 5);

        assert!(result.is_err());
    }
}
