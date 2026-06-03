use chrono::{NaiveDate, NaiveTime};
use crate::reminder::{Reminder, RepeatInterval};

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
            next_trigger: format_next_trigger(r),
        })
        .collect()
}

/// 格式化下次触发时间用于列表显示
fn format_next_trigger(r: &Reminder) -> Option<String> {
    let next = r.next_trigger?;
    let now = chrono::Local::now().date_naive();

    if next.date() == now {
        Some(next.format("%H:%M").to_string())
    } else if next.date() == now + chrono::Duration::days(1) {
        Some(format!("明天 {}", next.format("%H:%M")))
    } else {
        Some(next.format("%m-%d %H:%M").to_string())
    }
}

/// 获取重复类型的 ComboBox 索引
pub fn repeat_type_index(r: &Reminder) -> i32 {
    match &r.repeat {
        None => 0,
        Some(RepeatInterval::Minutes(_)) => 1,
        Some(RepeatInterval::Hours(_)) => 2,
        Some(RepeatInterval::Days(_)) => 3,
    }
}

/// 获取重复间隔的数值
pub fn repeat_amount(r: &Reminder) -> String {
    match &r.repeat {
        Some(RepeatInterval::Minutes(n)) => n.to_string(),
        Some(RepeatInterval::Hours(n)) => n.to_string(),
        Some(RepeatInterval::Days(n)) => n.to_string(),
        None => String::new(),
    }
}

/// 格式化日期为字符串
pub fn format_date(d: Option<NaiveDate>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
}

/// 格式化时间为字符串
pub fn format_time(t: Option<NaiveTime>) -> String {
    t.map(|t| t.format("%H:%M").to_string()).unwrap_or_default()
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
    fn to_reminder_items_shows_next_trigger() {
        use chrono::NaiveDateTime;

        let next = NaiveDateTime::parse_from_str("2026-06-04 15:00", "%Y-%m-%d %H:%M").unwrap();
        let reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(2))
            .with_next_trigger(next);

        let items = to_reminder_items(&[reminder]);

        assert!(items[0].next_trigger.is_some());
        let trigger_str = items[0].next_trigger.as_ref().unwrap();
        assert!(trigger_str.contains("15:00"), "应包含时间: {}", trigger_str);
    }

    #[test]
    fn repeat_type_index_returns_correct_value() {
        assert_eq!(repeat_type_index(&Reminder::new("a", "")), 0);

        let min = Reminder::new("a", "").with_repeat(RepeatInterval::Minutes(5));
        assert_eq!(repeat_type_index(&min), 1);

        let hr = Reminder::new("a", "").with_repeat(RepeatInterval::Hours(2));
        assert_eq!(repeat_type_index(&hr), 2);

        let day = Reminder::new("a", "").with_repeat(RepeatInterval::Days(1));
        assert_eq!(repeat_type_index(&day), 3);
    }

    #[test]
    fn format_date_some_and_none() {
        assert_eq!(format_date(Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap())), "2026-06-01");
        assert_eq!(format_date(None), "");
    }

    #[test]
    fn format_time_some_and_none() {
        assert_eq!(format_time(Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap())), "09:30");
        assert_eq!(format_time(None), "");
    }
}
