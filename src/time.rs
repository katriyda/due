use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use crate::reminder::{Reminder, RepeatInterval};

const FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
];

/// 解析本地时间字符串为 NaiveDateTime
pub fn parse_local_time(input: &str) -> Result<NaiveDateTime, String> {
    let input = input.trim();
    for fmt in FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, fmt) {
            return Ok(dt);
        }
    }
    if let Some(time) = try_parse_chinese_time(input) {
        let today = Local::now().date_naive();
        return Ok(today.and_time(time));
    }
    Err(format!("无法解析时间: '{}'", input))
}

/// 解析相对时间字符串，返回从当前时间起的 Duration
pub fn parse_relative_time(input: &str) -> Result<Duration, String> {
    let input = input.trim();
    let rest = input.strip_suffix("后").ok_or(format!("无法解析相对时间: '{}'", input))?;

    // 找到数字和单位的分界
    let num_end = rest.find(|c: char| !c.is_ascii_digit()).ok_or(format!("无法解析相对时间: '{}'", input))?;
    let (num_str, unit) = rest.split_at(num_end);
    let num: i64 = num_str.parse().map_err(|_| format!("无法解析数字: '{}'", num_str))?;

    match unit {
        "分钟" => Ok(Duration::minutes(num)),
        "小时" => Ok(Duration::hours(num)),
        "天" => Ok(Duration::days(num)),
        _ => Err(format!("未知时间单位: '{}'", unit)),
    }
}

/// 判断定时提醒是否应该触发（当前时间 >= 提醒时间）
pub fn should_trigger(scheduled_at: &NaiveDateTime, now: &NaiveDateTime) -> bool {
    now >= scheduled_at
}

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

    let mut next = next;
    while next <= *now {
        next = next_trigger_time(&interval, &next);
    }
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

/// 尝试解析中文时间格式，返回 NaiveTime
fn try_parse_chinese_time(input: &str) -> Option<NaiveTime> {
    let (is_pm, rest) = if let Some(r) = input.strip_prefix("下午") {
        (true, r)
    } else if let Some(r) = input.strip_prefix("上午") {
        (false, r)
    } else {
        return None;
    };

    // 按 "点" 分割：前面是小时，后面是分钟指示
    let dot_pos = rest.find('点')?;
    let hour_str = &rest[..dot_pos];
    let minute_part = &rest[dot_pos + "点".len()..];

    let minute = if minute_part == "半" { 30u32 } else { 0u32 };
    let hour: u32 = hour_str.parse().ok()?;
    let hour = if is_pm && hour < 12 { hour + 12 } else { hour };
    NaiveTime::from_hms_opt(hour, minute, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate, Timelike};

    #[test]
    fn parse_standard_local_time() {
        let result = parse_local_time("2026-06-03 15:00").unwrap();

        assert_eq!(result.year(), 2026);
        assert_eq!(result.month(), 6);
        assert_eq!(result.day(), 3);
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 0);
    }

    #[test]
    fn parse_local_time_with_seconds() {
        let result = parse_local_time("2026-06-03 15:00:30").unwrap();

        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 0);
        assert_eq!(result.second(), 30);
    }

    #[test]
    fn parse_chinese_time_afternoon() {
        let result = parse_local_time("下午3点").unwrap();

        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 0);
    }

    #[test]
    fn parse_chinese_time_morning_with_half() {
        let result = parse_local_time("上午10点半").unwrap();

        assert_eq!(result.hour(), 10);
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn parse_relative_time_minutes() {
        let duration = parse_relative_time("30分钟后").unwrap();

        assert_eq!(duration, Duration::minutes(30));
    }

    #[test]
    fn parse_relative_time_hours() {
        let duration = parse_relative_time("2小时后").unwrap();

        assert_eq!(duration, Duration::hours(2));
    }

    #[test]
    fn parse_relative_time_days() {
        let duration = parse_relative_time("1天后").unwrap();

        assert_eq!(duration, Duration::days(1));
    }

    #[test]
    fn trigger_when_time_has_passed() {
        let scheduled = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let now = NaiveDateTime::parse_from_str("2026-06-03 15:01", "%Y-%m-%d %H:%M").unwrap();

        assert!(should_trigger(&scheduled, &now));
    }

    #[test]
    fn no_trigger_when_time_has_not_passed() {
        let scheduled = NaiveDateTime::parse_from_str("2026-06-03 15:00", "%Y-%m-%d %H:%M").unwrap();
        let now = NaiveDateTime::parse_from_str("2026-06-03 14:59", "%Y-%m-%d %H:%M").unwrap();

        assert!(!should_trigger(&scheduled, &now));
    }

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
