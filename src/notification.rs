use log::{debug, error};

/// 发送系统通知
pub fn send(title: &str, content: &str) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("通知标题不能为空".to_string());
    }
    match notify_rust::Notification::new()
        .summary(title)
        .body(content)
        .show()
    {
        Ok(_) => {
            debug!("通知已发送: {}", title);
            Ok(())
        }
        Err(e) => {
            let msg = format!("通知发送失败: {}", e);
            error!("{}", msg);
            Err(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_notification_returns_ok() {
        let result = send("测试标题", "测试内容");

        assert!(result.is_ok(), "发送通知应该成功: {:?}", result);
    }

    #[test]
    fn send_with_empty_title_returns_error() {
        let result = send("", "测试内容");

        assert!(result.is_err(), "空标题应该返回错误");
    }

    #[test]
    fn send_with_reminder_fields() {
        let reminder = crate::reminder::Reminder::new("喝水", "每小时喝一杯水");
        // 验证 send() 可接受 &str 类型的 Reminder 字段
        // 通知可能因 Windows 系统限制失败，这里只验证不 panic
        let _ = send(&reminder.title, &reminder.content);
    }
}
