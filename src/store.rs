use std::cell::RefCell;
use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};
use crate::notification;
use crate::reminder::{self, Reminder, RepeatInterval};
use crate::time;

pub struct ReminderStore {
    reminders: RefCell<Vec<Reminder>>,
    dir: PathBuf,
    window: slint::Weak<crate::MainWindow>,
}

impl ReminderStore {
    pub fn new(dir: PathBuf, window: slint::Weak<crate::MainWindow>) -> Self {
        let reminders = reminder::load_reminders(&dir).unwrap_or_default();
        Self {
            reminders: RefCell::new(reminders),
            dir,
            window,
        }
    }

    pub fn reminders(&self) -> std::cell::Ref<'_, Vec<Reminder>> {
        self.reminders.borrow()
    }

    /// 添加新提醒（带默认间隔）
    /// interval_idx: 0=5分钟, 1=15分钟, 2=30分钟, 3=1小时, 4=1天
    pub fn add(&self, title: &str, content: &str, interval_idx: i32) {
        let interval = match interval_idx {
            0 => RepeatInterval::Minutes(5),
            1 => RepeatInterval::Minutes(15),
            2 => RepeatInterval::Minutes(30),
            3 => RepeatInterval::Hours(1),
            4 => RepeatInterval::Days(1),
            _ => RepeatInterval::Minutes(30),
        };

        let now = chrono::Local::now().naive_local();
        let next = time::next_trigger_time(&interval, &now);

        let mut r = Reminder::new(title, content);
        r.repeat = Some(interval);
        r.next_trigger = Some(next);

        self.reminders.borrow_mut().push(r);
        self.persist();
    }

    /// 删除提醒
    pub fn delete(&self, index: usize) -> Result<(), String> {
        let mut reminders = self.reminders.borrow_mut();
        if index >= reminders.len() {
            return Err(format!("索引越界: {} (共 {} 条)", index, reminders.len()));
        }
        reminders.remove(index);
        drop(reminders);
        self.persist();
        Ok(())
    }

    /// 切换启用状态
    pub fn toggle(&self, index: usize) -> Result<(), String> {
        let mut reminders = self.reminders.borrow_mut();
        if index >= reminders.len() {
            return Err(format!("索引越界: {} (共 {} 条)", index, reminders.len()));
        }
        reminders[index].enabled = !reminders[index].enabled;
        drop(reminders);
        self.persist();
        Ok(())
    }

    /// 从 window 编辑面板读取值并保存到指定提醒
    pub fn save_edit(&self, index: usize) -> Result<(), String> {
        let w = self.window.upgrade().ok_or("窗口已关闭")?;

        self.apply_edit_data(
            index,
            &w.get_edit_title().to_string(),
            &w.get_edit_content().to_string(),
            w.get_repeat_type_index(),
            &w.get_repeat_amount_value().to_string(),
            &w.get_edit_start_date().to_string(),
            &w.get_edit_end_date().to_string(),
            &w.get_edit_daily_start().to_string(),
            &w.get_edit_daily_end().to_string(),
            &w.get_repeat_limit_value().to_string(),
        )
    }

    /// 将编辑数据应用到指定提醒（可独立测试）
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_edit_data(
        &self,
        index: usize,
        title: &str,
        content: &str,
        repeat_type_idx: i32,
        repeat_amount_str: &str,
        start_date_str: &str,
        end_date_str: &str,
        daily_start_str: &str,
        daily_end_str: &str,
        repeat_limit_str: &str,
    ) -> Result<(), String> {
        let mut reminders = self.reminders.borrow_mut();
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

        // 解析重复次数限制
        let limit_str = repeat_limit_str.trim();
        r.repeat_limit = if limit_str.is_empty() {
            None
        } else {
            let limit: u32 = limit_str
                .parse()
                .map_err(|_| format!("无效的次数限制: '{}'", limit_str))?;
            Some(limit)
        };

        // 解析日期范围
        r.start_date = parse_optional_date(start_date_str)?;
        r.end_date = parse_optional_date(end_date_str)?;

        // 解析每日时间窗口
        r.daily_start = parse_optional_time(daily_start_str)?;
        r.daily_end = parse_optional_time(daily_end_str)?;

        drop(reminders);
        self.persist();
        Ok(())
    }

    /// 填充编辑面板字段
    pub fn select(&self, index: usize) {
        let reminders = self.reminders.borrow();
        if let Some(r) = reminders.get(index) {
            if let Some(w) = self.window.upgrade() {
                w.set_selected_index(index as i32);
                w.set_edit_title(r.title.clone().into());
                w.set_edit_content(r.content.clone().into());
                w.set_repeat_type_index(crate::ui::repeat_type_index(r));
                w.set_edit_start_date(crate::ui::format_date(r.start_date).into());
                w.set_edit_end_date(crate::ui::format_date(r.end_date).into());
                w.set_edit_daily_start(crate::ui::format_time(r.daily_start).into());
                w.set_edit_daily_end(crate::ui::format_time(r.daily_end).into());
                w.set_repeat_amount_value(crate::ui::repeat_amount(r).into());
                w.set_repeat_limit_value(
                    r.repeat_limit
                        .map(|l| l.to_string())
                        .unwrap_or_default()
                        .into(),
                );
            }
        }
    }

    /// 调度器：检查触发、发送通知、更新状态
    pub fn tick(&self) {
        let now = chrono::Local::now().naive_local();
        let mut reminders = self.reminders.borrow_mut();
        let mut changed = false;

        for r in reminders.iter_mut() {
            if time::should_trigger_recurring(r, &now) {
                let _ = notification::send(&r.title, &r.content);
                time::update_after_trigger(r, &now);
                changed = true;
            }
        }

        if changed {
            drop(reminders);
            self.persist();
            self.refresh_ui();
        }
    }

    fn refresh_ui(&self) {
        if let Some(w) = self.window.upgrade() {
            let items: Vec<crate::ReminderItem> = crate::ui::to_reminder_items(&self.reminders.borrow())
                .iter()
                .map(|item| crate::ReminderItem {
                    id: item.id as i32,
                    title: item.title.clone().into(),
                    content: item.content.clone().into(),
                    enabled: item.enabled,
                    completed: item.completed,
                    next_trigger: item.next_trigger.clone().unwrap_or_default().into(),
                })
                .collect();
            let model = std::rc::Rc::new(slint::VecModel::from(items));
            w.set_reminders(model.into());
        }
    }

    fn persist(&self) {
        let _ = reminder::save_reminders(&self.dir, &self.reminders.borrow());
    }
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

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("due_store_test_{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn new_store_starts_empty_when_no_file() {
        let dir = temp_dir("new_empty");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        assert_eq!(store.reminders().len(), 0);
    }

    #[test]
    fn new_store_loads_existing_reminders() {
        let dir = temp_dir("new_load");
        let existing = vec![
            Reminder::new("喝水", "每小时喝一杯水"),
            Reminder::new("站立", "久坐后站起来活动"),
        ];
        reminder::save_reminders(&dir, &existing).unwrap();

        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        assert_eq!(store.reminders().len(), 2);
        assert_eq!(store.reminders()[0].title, "喝水");
        assert_eq!(store.reminders()[1].title, "站立");
    }

    #[test]
    fn add_creates_reminder_with_interval_and_next_trigger() {
        let dir = temp_dir("add_basic");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        store.add("喝水", "每小时喝一杯水", 3); // 3 = 1小时

        let reminders = store.reminders();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].title, "喝水");
        assert_eq!(reminders[0].content, "每小时喝一杯水");
        assert_eq!(reminders[0].repeat, Some(RepeatInterval::Hours(1)));
        assert!(reminders[0].next_trigger.is_some(), "next_trigger 应已设置");
    }

    #[test]
    fn add_persists_to_file() {
        let dir = temp_dir("add_persist");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        store.add("喝水", "", 2); // 30分钟

        let loaded = reminder::load_reminders(&dir).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "喝水");
    }

    #[test]
    fn add_multiple_reminders() {
        let dir = temp_dir("add_multiple");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        store.add("喝水", "", 2);
        store.add("站立", "", 3);
        store.add("买菜", "鸡蛋", 0);

        assert_eq!(store.reminders().len(), 3);
        let loaded = reminder::load_reminders(&dir).unwrap();
        assert_eq!(loaded.len(), 3);
    }

    #[test]
    fn delete_removes_item() {
        let dir = temp_dir("delete_basic");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);
        store.add("站立", "", 3);

        store.delete(0).unwrap();

        assert_eq!(store.reminders().len(), 1);
        assert_eq!(store.reminders()[0].title, "站立");
    }

    #[test]
    fn delete_persists_to_file() {
        let dir = temp_dir("delete_persist");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);
        store.add("站立", "", 3);

        store.delete(0).unwrap();

        let loaded = reminder::load_reminders(&dir).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "站立");
    }

    #[test]
    fn delete_out_of_bounds_returns_error() {
        let dir = temp_dir("delete_oob");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        let result = store.delete(5);

        assert!(result.is_err());
        assert_eq!(store.reminders().len(), 1);
    }

    #[test]
    fn toggle_flips_enabled_state() {
        let dir = temp_dir("toggle_basic");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        store.toggle(0).unwrap();
        assert!(!store.reminders()[0].enabled);

        store.toggle(0).unwrap();
        assert!(store.reminders()[0].enabled);
    }

    #[test]
    fn toggle_persists_to_file() {
        let dir = temp_dir("toggle_persist");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        store.toggle(0).unwrap();

        let loaded = reminder::load_reminders(&dir).unwrap();
        assert!(!loaded[0].enabled);
    }

    #[test]
    fn toggle_out_of_bounds_returns_error() {
        let dir = temp_dir("toggle_oob");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        let result = store.toggle(0);

        assert!(result.is_err());
    }

    #[test]
    fn apply_edit_data_updates_all_fields() {
        let dir = temp_dir("edit_basic");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("旧标题", "旧内容", 2);

        store.apply_edit_data(
            0, "新标题", "新内容", 2, "3",
            "2026-06-01", "2026-12-31", "09:00", "18:00", "5",
        ).unwrap();

        let r = &store.reminders()[0];
        assert_eq!(r.title, "新标题");
        assert_eq!(r.content, "新内容");
        assert_eq!(r.repeat, Some(RepeatInterval::Hours(3)));
        assert_eq!(r.start_date, Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
        assert_eq!(r.end_date, Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()));
        assert_eq!(r.daily_start, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(r.daily_end, Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()));
        assert_eq!(r.repeat_limit, Some(5));
    }

    #[test]
    fn apply_edit_data_clears_repeat_when_type_is_none() {
        let dir = temp_dir("edit_clear");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 3);

        store.apply_edit_data(0, "喝水", "", 0, "", "", "", "", "", "").unwrap();

        assert_eq!(store.reminders()[0].repeat, None);
    }

    #[test]
    fn apply_edit_data_invalid_index_returns_error() {
        let dir = temp_dir("edit_oob");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        let result = store.apply_edit_data(0, "x", "", 0, "", "", "", "", "", "");

        assert!(result.is_err());
    }

    #[test]
    fn apply_edit_data_invalid_date_returns_error() {
        let dir = temp_dir("edit_bad_date");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        let result = store.apply_edit_data(0, "喝水", "", 0, "", "not-a-date", "", "", "", "");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效日期格式"));
    }

    #[test]
    fn apply_edit_data_persists_to_file() {
        let dir = temp_dir("edit_persist");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        store.apply_edit_data(0, "新标题", "", 0, "", "", "", "", "", "").unwrap();

        let loaded = reminder::load_reminders(&dir).unwrap();
        assert_eq!(loaded[0].title, "新标题");
    }

    #[test]
    fn apply_edit_data_empty_amount_for_repeat_returns_error() {
        let dir = temp_dir("edit_empty_amount");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        let result = store.apply_edit_data(0, "喝水", "", 2, "", "", "", "", "", "");

        assert!(result.is_err());
    }

    #[test]
    fn select_does_nothing_without_window() {
        let dir = temp_dir("select_no_window");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());
        store.add("喝水", "", 2);

        // 不应 panic，window 为 None 时静默跳过
        store.select(0);
    }

    #[test]
    fn tick_updates_next_trigger_when_due() {
        let dir = temp_dir("tick_basic");
        let store = ReminderStore::new(dir.clone(), slint::Weak::default());

        // 手动构造一个 next_trigger 在过去的提醒
        let mut r = Reminder::new("喝水", "").with_repeat(RepeatInterval::Hours(1));
        r.next_trigger = Some(chrono::NaiveDateTime::parse_from_str("2020-01-01 00:00", "%Y-%m-%d %H:%M").unwrap());
        store.reminders.borrow_mut().push(r);

        store.tick();

        let reminders = store.reminders();
        assert!(reminders[0].next_trigger.is_some());
        assert!(reminders[0].next_trigger.unwrap() > chrono::NaiveDateTime::parse_from_str("2020-01-01 00:00", "%Y-%m-%d %H:%M").unwrap());
    }
}
