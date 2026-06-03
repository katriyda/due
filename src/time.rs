use chrono::{Duration, NaiveDateTime};
use log::debug;
use crate::reminder::{Reminder, RepeatInterval};

/// 计算重复提醒的下次触发时间
pub fn next_trigger_time(
    interval: &RepeatInterval,
    after: &NaiveDateTime,
) -> NaiveDateTime {
    match interval {
        RepeatInterval::Minutes(n) => *after + Duration::minutes(*n as i64),
        RepeatInterval::Hours(n) => *after + Duration::hours(*n as i64),
        RepeatInterval::Days(n) => *after + Duration::days(*n as i64),
    }
}

/// 检查提醒是否在日期范围内
pub fn is_within_date_range(reminder: &Reminder, now: &NaiveDateTime) -> bool {
    let today = now.date();

    if let Some(start) = reminder.start_date {
        if today < start {
            return false;
        }
    }

    if let Some(end) = reminder.end_date {
        if today > end {
            return false;
        }
    }

    true
}

/// 检查提醒是否在每日活跃时段内
pub fn is_within_daily_window(reminder: &Reminder, now: &NaiveDateTime) -> bool {
    let current_time = now.time();

    if let (Some(start), Some(end)) = (reminder.daily_start, reminder.daily_end) {
        return current_time >= start && current_time <= end;
    }

    true
}

/// 检查重复提醒是否达到次数限制
pub fn has_reached_limit(reminder: &Reminder) -> bool {
    if let Some(limit) = reminder.repeat_limit {
        return reminder.repeat_count >= limit;
    }

    false
}

/// 重启时将 next_trigger 推进到未来时间点（不补触发，只找下一个未来触发点）
pub fn advance_next_trigger(reminder: &mut Reminder, now: &NaiveDateTime) {
    let next = match reminder.next_trigger {
        Some(n) => n,
        None => return,
    };

    if next > *now {
        return;
    }

    let interval = match &reminder.repeat {
        Some(i) => i.clone(),
        None => return,
    };

    let old_next = next;
    let mut next = next;
    while next <= *now {
        next = next_trigger_time(&interval, &next);
    }
    debug!("推进提醒触发时间: {}，从 {} 到 {}", reminder.title, old_next, next);
    reminder.next_trigger = Some(next);
}

/// 触发后更新提醒状态：更新 next_trigger、repeat_count，一次性提醒自动完成
pub fn update_after_trigger(reminder: &mut Reminder, now: &NaiveDateTime) {
    reminder.repeat_count += 1;

    if let Some(limit) = reminder.repeat_limit {
        if reminder.repeat_count >= limit {
            reminder.completed = true;
            reminder.next_trigger = None;
            return;
        }
    }

    if let Some(interval) = &reminder.repeat {
        reminder.next_trigger = Some(next_trigger_time(interval, now));
    }
}

/// 综合判断重复提醒是否应该触发
pub fn should_trigger_recurring(reminder: &Reminder, now: &NaiveDateTime) -> bool {
    if !reminder.enabled || reminder.completed {
        return false;
    }

    if has_reached_limit(reminder) {
        return false;
    }

    if let Some(next) = reminder.next_trigger {
        if *now < next {
            return false;
        }
    }

    if !is_within_date_range(reminder, now) {
        return false;
    }

    if !is_within_daily_window(reminder, now) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, Timelike};

    #[test]
    fn next_trigger_after_30_minutes() {
        let after = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let interval = RepeatInterval::Minutes(30);

        let next = next_trigger_time(&interval, &after);

        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn next_trigger_after_2_hours() {
        let after = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let interval = RepeatInterval::Hours(2);

        let next = next_trigger_time(&interval, &after);

        assert_eq!(next.hour(), 17);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn within_date_range_when_no_dates_set() {
        let reminder = Reminder::new("喝水", "每小时喝一杯水");
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(is_within_date_range(&reminder, &now));
    }

    #[test]
    fn within_date_range_when_between_dates() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.start_date = Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        reminder.end_date = Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
        let now = NaiveDateTime::parse_from_str("2026-06-15 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(is_within_date_range(&reminder, &now));
    }

    #[test]
    fn outside_date_range_when_before_start() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.start_date = Some(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(!is_within_date_range(&reminder, &now));
    }

    #[test]
    fn outside_date_range_when_after_end() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.end_date = Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        let now = NaiveDateTime::parse_from_str("2026-06-15 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(!is_within_date_range(&reminder, &now));
    }

    #[test]
    fn within_daily_window_when_no_window_set() {
        let reminder = Reminder::new("喝水", "每小时喝一杯水");
        let now = NaiveDateTime::parse_from_str("2026-06-03 03:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(is_within_daily_window(&reminder, &now));
    }

    #[test]
    fn within_daily_window_when_in_range() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.daily_start = Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        reminder.daily_end = Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap());
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(is_within_daily_window(&reminder, &now));
    }

    #[test]
    fn outside_daily_window_when_before_start() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.daily_start = Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        reminder.daily_end = Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap());
        let now = NaiveDateTime::parse_from_str("2026-06-03 08:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(!is_within_daily_window(&reminder, &now));
    }

    #[test]
    fn has_reached_limit_when_count_exceeds() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.repeat_limit = Some(5);
        reminder.repeat_count = 5;

        assert!(has_reached_limit(&reminder));
    }

    #[test]
    fn has_not_reached_limit_when_count_below() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.repeat_limit = Some(5);
        reminder.repeat_count = 3;

        assert!(!has_reached_limit(&reminder));
    }

    #[test]
    fn has_not_reached_limit_when_no_limit_set() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.repeat_count = 100;

        assert!(!has_reached_limit(&reminder));
    }

    #[test]
    fn should_not_trigger_when_completed() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.completed = true;
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(!should_trigger_recurring(&reminder, &now));
    }

    #[test]
    fn should_not_trigger_when_limit_reached() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水");
        reminder.repeat_limit = Some(5);
        reminder.repeat_count = 5;
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(!should_trigger_recurring(&reminder, &now));
    }

    #[test]
    fn should_trigger_when_all_conditions_met() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1));
        reminder.daily_start = Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        reminder.daily_end = Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap());
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(should_trigger_recurring(&reminder, &now));
    }

    #[test]
    fn should_not_trigger_when_next_trigger_in_future() {
        let next = NaiveDateTime::parse_from_str("2026-06-03 16:00", "%Y-%m-%d %H:%M").unwrap();
        let reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1))
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(!should_trigger_recurring(&reminder, &now));
    }

    #[test]
    fn should_trigger_when_next_trigger_reached() {
        let next = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1))
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(should_trigger_recurring(&reminder, &now));
    }

    #[test]
    fn should_trigger_when_next_trigger_past() {
        let next = NaiveDateTime::parse_from_str("2026-06-03 14:00", "%Y-%m-%d %H:%M").unwrap();
        let reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1))
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        assert!(should_trigger_recurring(&reminder, &now));
    }

    #[test]
    fn update_after_trigger_sets_next_trigger() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1));
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        update_after_trigger(&mut reminder, &now);

        assert_eq!(reminder.next_trigger, Some(NaiveDateTime::parse_from_str("2026-06-03 16:00", "%Y-%m-%d %H:%M").unwrap()));
        assert_eq!(reminder.repeat_count, 1);
        assert!(!reminder.completed);
    }

    #[test]
    fn update_after_trigger_increments_count() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1));
        reminder.repeat_count = 3;
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        update_after_trigger(&mut reminder, &now);

        assert_eq!(reminder.repeat_count, 4);
    }

    #[test]
    fn update_after_trigger_completes_when_limit_reached() {
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1));
        reminder.repeat_limit = Some(1);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        update_after_trigger(&mut reminder, &now);

        assert!(reminder.completed);
        assert_eq!(reminder.next_trigger, None);
        assert_eq!(reminder.repeat_count, 1);
    }

    #[test]
    fn update_after_trigger_one_time_reminder_auto_completes() {
        let mut reminder = Reminder::new("买菜", "鸡蛋、牛奶")
            .with_repeat(RepeatInterval::Minutes(1));
        reminder.repeat_limit = Some(1);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        update_after_trigger(&mut reminder, &now);

        assert!(reminder.completed, "一次性提醒触发后应自动完成");
        assert_eq!(reminder.next_trigger, None);
    }

    #[test]
    fn advance_next_trigger_does_nothing_when_in_future() {
        let next = NaiveDateTime::parse_from_str("2026-06-03 16:00", "%Y-%m-%d %H:%M").unwrap();
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_repeat(RepeatInterval::Hours(1))
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();

        advance_next_trigger(&mut reminder, &now);

        assert_eq!(reminder.next_trigger, Some(next));
    }

    #[test]
    fn advance_next_trigger_skips_missed_triggers() {
        // next_trigger was 15:00, now is 15:20, interval 30min → should advance to 15:30
        let next = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let mut reminder = Reminder::new("喝水", "每半小时喝水")
            .with_repeat(RepeatInterval::Minutes(30))
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:20", "%Y-%m-%d %H:%M").unwrap();

        advance_next_trigger(&mut reminder, &now);

        let expected = NaiveDateTime::parse_from_str("2026-06-03 15:30", "%Y-%m-%d %H:%M").unwrap();
        assert_eq!(reminder.next_trigger, Some(expected));
    }

    #[test]
    fn advance_next_trigger_skips_multiple_missed() {
        // next_trigger was 15:00, now is 16:05, interval 30min → should advance to 16:30
        let next = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let mut reminder = Reminder::new("喝水", "每半小时喝水")
            .with_repeat(RepeatInterval::Minutes(30))
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 16:05", "%Y-%m-%d %H:%M").unwrap();

        advance_next_trigger(&mut reminder, &now);

        let expected = NaiveDateTime::parse_from_str("2026-06-03 16:30", "%Y-%m-%d %H:%M").unwrap();
        assert_eq!(reminder.next_trigger, Some(expected));
    }

    #[test]
    fn advance_next_trigger_does_nothing_without_repeat() {
        let next = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let mut reminder = Reminder::new("喝水", "每小时喝一杯水")
            .with_next_trigger(next);
        let now = NaiveDateTime::parse_from_str("2026-06-03 16:00", "%Y-%m-%d %H:%M").unwrap();

        advance_next_trigger(&mut reminder, &now);

        assert_eq!(reminder.next_trigger, Some(next), "没有 repeat 的提醒不应推进");
    }
}
