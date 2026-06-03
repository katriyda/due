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
            next_trigger: format_repeat_info(r),
        })
        .collect()
}

/// 格式化重复信息用于列表显示
fn format_repeat_info(r: &Reminder) -> Option<String> {
    match &r.repeat {
        Some(RepeatInterval::Minutes(n)) => Some(format!("每 {} 分钟", n)),
        Some(RepeatInterval::Hours(n)) => Some(format!("每 {} 小时", n)),
        Some(RepeatInterval::Days(n)) => Some(format!("每 {} 天", n)),
        None => None,
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

/// 保存编辑后的提醒
#[allow(clippy::too_many_arguments)]
pub fn save_reminder(
    reminders: &mut Vec<Reminder>,
    index: usize,
    title: &str,
    content: &str,
    repeat_type_idx: i32,
    repeat_amount_str: &str,
    start_date_str: &str,
    end_date_str: &str,
    daily_start_str: &str,
    daily_end_str: &str,
) -> Result<(), String> {
    if index >= reminders.len() {
        return Err(format!("索引越界: {} (共 {} 条)", index, reminders.len()));
    }

    let r = &mut reminders[index];
    r.title = title.to_string();
    r.content = content.to_string();

    // 解析重复设置
    r.repeat = match repeat_type_idx {
        1 | 2 | 3 => {
            let amount: u32 = repeat_amount_str
                .trim()
                .parse()
                .map_err(|_| format!("无效的间隔数值: '{}'", repeat_amount_str))?;
            if amount == 0 {
                return Err("间隔数值不能为 0".to_string());
            }
            Some(match repeat_type_idx {
                1 => RepeatInterval::Minutes(amount),
                2 => RepeatInterval::Hours(amount),
                3 => RepeatInterval::Days(amount),
                _ => unreachable!(),
            })
        }
        _ => None,
    };

    // 解析日期范围
    r.start_date = parse_optional_date(start_date_str)?;
    r.end_date = parse_optional_date(end_date_str)?;

    // 解析每日时间窗口
    r.daily_start = parse_optional_time(daily_start_str)?;
    r.daily_end = parse_optional_time(daily_end_str)?;

    Ok(())
}

fn parse_optional_date(s: &str) -> Result<Option<NaiveDate>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| format!("无效日期格式: '{}'（应为 YYYY-MM-DD）", s))
}

fn parse_optional_time(s: &str) -> Result<Option<NaiveTime>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    NaiveTime::parse_from_str(s, "%H:%M")
        .map(Some)
        .map_err(|_| format!("无效时间格式: '{}'（应为 HH:MM）", s))
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
    fn to_reminder_items_shows_repeat_info() {
        let reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(2));

        let items = to_reminder_items(&[reminder]);

        assert_eq!(items[0].next_trigger, Some("每 2 小时".to_string()));
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

    #[test]
    fn save_reminder_updates_all_fields() {
        let mut reminders = vec![Reminder::new("喝水", "旧内容")];

        save_reminder(
            &mut reminders,
            0,
            "新标题",
            "新内容",
            2,    // 按小时
            "3",  // 每3小时
            "2026-06-01",
            "2026-12-31",
            "09:00",
            "18:00",
        )
        .unwrap();

        let r = &reminders[0];
        assert_eq!(r.title, "新标题");
        assert_eq!(r.content, "新内容");
        assert_eq!(r.repeat, Some(RepeatInterval::Hours(3)));
        assert_eq!(r.start_date, Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
        assert_eq!(r.end_date, Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()));
        assert_eq!(r.daily_start, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(r.daily_end, Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()));
    }

    #[test]
    fn save_reminder_clears_repeat_when_type_is_none() {
        let mut reminders = vec![
            Reminder::new("喝水", "每小时喝一杯水")
                .with_repeat(RepeatInterval::Hours(1)),
        ];

        save_reminder(&mut reminders, 0, "喝水", "", 0, "", "", "", "", "").unwrap();

        assert_eq!(reminders[0].repeat, None);
    }

    #[test]
    fn save_reminder_invalid_index_returns_error() {
        let mut reminders = vec![Reminder::new("喝水", "")];

        let result = save_reminder(&mut reminders, 5, "x", "", 0, "", "", "", "", "");

        assert!(result.is_err());
    }

    #[test]
    fn save_reminder_invalid_date_returns_error() {
        let mut reminders = vec![Reminder::new("喝水", "")];

        let result = save_reminder(&mut reminders, 0, "喝水", "", 0, "", "not-a-date", "", "", "");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效日期格式"));
    }

    #[test]
    fn save_reminder_empty_amount_for_repeat_returns_error() {
        let mut reminders = vec![Reminder::new("喝水", "")];

        let result = save_reminder(&mut reminders, 0, "喝水", "", 2, "", "", "", "", "");

        assert!(result.is_err());
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
