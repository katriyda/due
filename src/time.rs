use chrono::{Duration, Local, NaiveDateTime, NaiveTime};

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
    use chrono::{Datelike, Timelike};

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
}
