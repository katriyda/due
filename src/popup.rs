use chrono::{Duration, NaiveDateTime};
use crate::reminder::Reminder;

/// 弹窗用户操作
#[derive(Debug, Clone, PartialEq)]
pub enum PopupAction {
    /// 关闭弹窗
    Dismiss,
    /// 延后（分钟）
    Snooze(u32),
    /// 标记完成
    Complete,
}

/// 弹窗配置
#[derive(Debug, Clone)]
pub struct PopupConfig {
    /// 自动关闭秒数
    pub auto_close_secs: u32,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            auto_close_secs: 30,
        }
    }
}

/// 返回可用的延后选项（分钟）
pub fn snooze_options() -> Vec<u32> {
    vec![5, 15, 30, 60]
}

/// 处理弹窗操作，返回对提醒的下次触发时间（仅延后操作有值）
pub fn handle_action(
    action: &PopupAction,
    reminder: &mut Reminder,
    now: &NaiveDateTime,
) -> Option<NaiveDateTime> {
    match action {
        PopupAction::Dismiss => None,
        PopupAction::Snooze(minutes) => {
            let next = *now + Duration::minutes(*minutes as i64);
            Some(next)
        }
        PopupAction::Complete => {
            reminder.completed = true;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn snooze_options_returns_four_choices() {
        let options = snooze_options();

        assert_eq!(options, vec![5, 15, 30, 60]);
    }

    #[test]
    fn dismiss_does_not_change_reminder() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        let result = handle_action(&PopupAction::Dismiss, &mut reminder, &now);

        assert!(result.is_none());
        assert!(!reminder.completed);
    }

    #[test]
    fn complete_marks_reminder_as_completed() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        let result = handle_action(&PopupAction::Complete, &mut reminder, &now);

        assert!(result.is_none());
        assert!(reminder.completed);
    }

    #[test]
    fn snooze_returns_next_trigger_time() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        let result = handle_action(&PopupAction::Snooze(15), &mut reminder, &now);

        assert!(result.is_some());
        let next = result.unwrap();
        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 15);
    }

    #[test]
    fn default_popup_config_has_30s_timeout() {
        let config = PopupConfig::default();

        assert_eq!(config.auto_close_secs, 30);
    }
}
